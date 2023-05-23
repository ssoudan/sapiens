//! Main for sapiens_exp

use std::fmt::Display;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use dotenvy::dotenv_override;
use sapiens::models::SupportedModel;
use sapiens::{models, run_to_the_end, wrap_observer, ChainType};
use sapiens_exp::evaluate::Trial;
use sapiens_exp::tools::scenario_0;
use sapiens_exp::traces::TraceObserver;
use sapiens_exp::{setup, Config};
use tracing::{error, info, trace};
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
    /// The type of chain to use
    #[arg(long, default_value_t = ChainType::SingleStepOODA, value_enum, env)]
    chain: ChainType,

    /// Model to use
    #[arg(long, default_value_t = SupportedModel::GPT3_5Turbo, value_enum, env)]
    model: SupportedModel,

    /// Maximum number of steps to execute
    #[arg(short, long, default_value_t = 10)]
    max_steps: usize,

    /// Minimum tokens for completion
    #[arg(long, default_value_t = 256)]
    min_tokens_for_completion: usize,

    /// Max tokens for the model to generate
    #[arg(long)]
    max_tokens: Option<usize>,

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
            chain: args.chain,
            model: args.model.clone(),
            max_steps: args.max_steps,
            min_tokens_for_completion: args.min_tokens_for_completion,
            max_tokens: args.max_tokens,
            temperature: Some(args.temperature),
            scenario: args.scenario.to_string(),
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
    let trial_config = Config::from(&args);

    info!(trial_config = ?trial_config, "Starting sapiens_exp");

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

    let temperature = Some(args.temperature);

    let model = match args.model {
        SupportedModel::ChatBison001 => {
            let google_api_key =
                std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY is not set");

            models::vertex_ai::build(google_api_key, temperature)
                .await
                .expect("Failed to build model")
        }
        _ => {
            let api_key = std::env::var("OPENAI_API_KEY").ok();
            let api_base = std::env::var("OPENAI_API_BASE").ok();

            models::openai::build(args.model.clone(), api_key, api_base, temperature)
                .await
                .expect("Failed to build model")
        }
    };

    let config = sapiens::SapiensConfig {
        max_steps: args.max_steps,
        chain_type: args.chain,
        model,
        min_tokens_for_completion: args.min_tokens_for_completion,
        max_tokens: args.max_tokens,
    };

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

    match run_to_the_end(config, toolbox.clone(), task.clone(), w_trace_observer).await {
        Ok(_) => {
            info!("Task completed");
        }
        Err(e) => {
            error!(error = ?e, "Task failed");
        }
    }

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
        trial_config,
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
