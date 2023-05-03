//! Main for sapiens_exp

use std::fmt::Display;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use dotenvy::dotenv_override;
use sapiens::context::{ChatEntry, ChatEntryFormatter};
use sapiens::openai::Role;
use sapiens::{run_to_the_end, wrap_observer};
use sapiens_exp::evaluate::Trial;
use sapiens_exp::tools::scenario_0;
use sapiens_exp::traces::TraceObserver;
use sapiens_exp::{setup, Config};
use tracing::{info, trace};
use tracing_subscriber::EnvFilter;

// FUTURE(ssoudan) BO

/// A scenario to execute
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Default)]
enum Scenario {
    /// Scenario 0 - make a cereal bowl with milk
    #[default]
    Scenario0,
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scenario::Scenario0 => write!(f, "Scenario0"),
        }
    }
}

/// A bot that can do things - or at least try to.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Model to use
    #[arg(long, default_value = "gpt-3.5-turbo")]
    model: String,

    /// Maximum number of steps to execute
    #[arg(short, long, default_value_t = 10)]
    max_steps: usize,

    /// Task to execute
    #[arg(short, long, default_value = "Make me a bowl of cereal with milk")]
    task: String,

    /// Scenario to execute
    #[arg(short, long, value_enum, default_value_t = Scenario::Scenario0)]
    scenario: Scenario,

    /// Experiments folder
    #[arg(long, default_value = "experiments/data/")]
    experiments_folder: String,

    /// File to save the trial record. Default to trial_<random>.json
    #[arg(long)]
    trial_file: Option<String>,

    /// Temperature for the model sampling
    /// min: 0, max: 2
    /// The higher the temperature, the crazier the text.
    #[arg(long, default_value_t = 0.)]
    temperature: f32,
}

impl From<&Args> for Config {
    fn from(args: &Args) -> Self {
        Self {
            model: args.model.clone(),
            max_steps: args.max_steps,
            temperature: Some(args.temperature),
            scenario: args.scenario.to_string(),
        }
    }
}

struct ColorFormatter;

impl ChatEntryFormatter for ColorFormatter {
    fn format(&self, entry: &ChatEntry) -> String {
        let msg = &entry.msg;
        let role = &entry.role;
        match role {
            Role::System => msg.yellow().to_string(),
            Role::User => msg.green().to_string(),
            Role::Assistant => msg.blue().to_string(),
        }
    }
}

#[pyo3_asyncio::tokio::main]
async fn main() -> Result<(), pyo3::PyErr> {
    let args = Args::parse();

    let _ = dotenv_override();

    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_default())
        .init();
    // Prepare config
    let config = Config::from(&args);

    info!(config = ?config, "Starting sapiens_exp");

    let trial_file = args
        .trial_file
        .clone()
        .unwrap_or_else(|| format!("trial_{}.json", rand::random::<u64>()));

    let experiments_folder = args.experiments_folder.clone();
    let trial_path = std::path::Path::new(&experiments_folder).join(&trial_file);

    info!("Going to save trials in {} ", trial_path.to_str().unwrap());

    let toolbox = setup::basic_toolbox().await;

    // prepare scenario
    let (toolbox, shared_state) = match args.scenario {
        Scenario::Scenario0 => scenario_0::build(toolbox).await,
    };

    // reset stats
    toolbox.reset_stats().await;

    let openai_client = sapiens::openai::Client::new().with_api_key(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set in configuration file"),
    );

    // Sanitation
    // remove environment variables that could be used to access the host
    for (k, _) in std::env::vars() {
        std::env::remove_var(&k);
    }
    assert!(
        std::env::vars().next().is_none(),
        "Environment is not empty"
    );

    let trace_observer = TraceObserver::new(shared_state.clone());
    let trace_observer = wrap_observer(trace_observer);
    let w_trace_observer = Arc::downgrade(&trace_observer);

    let task = args.task.clone();
    let _ = run_to_the_end(
        toolbox.clone(),
        openai_client,
        (&config).into(),
        task.clone(),
        w_trace_observer,
    )
    .await;

    let trace = { trace_observer.lock().await.trace().await };

    // Collect tool utilization stats
    let tool_stats = toolbox.stats().await;

    // Has the task been completed?
    let reached_accepting_state = {
        let guard = shared_state.lock().await;
        guard.has_reached_accepting_state()
    };

    // What is the final state name?
    let final_state_name = {
        let guard = shared_state.lock().await;
        guard.state()
    };

    // Build trial
    let trial = Trial::build(
        config,
        task,
        trace,
        tool_stats,
        reached_accepting_state,
        final_state_name,
    );

    trace!(trial = ?trial, "Trial");

    // Save to {experiments_folder}/trial.json
    let _ = std::fs::create_dir_all(&experiments_folder);
    let _ = std::fs::write(&trial_path, serde_json::to_string_pretty(&trial).unwrap());

    info!("Trial saved to {}", trial_path.to_str().unwrap());

    Ok(())
}
