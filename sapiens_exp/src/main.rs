//! Main for sapiens_exp

use std::sync::Arc;

use clap::Parser;
use colored::Colorize;
use dotenvy::dotenv_override;
use sapiens::context::{ChatEntry, ChatEntryFormatter};
use sapiens::openai::Role;
use sapiens::{run_to_the_end, wrap_observer};
use sapiens_exp::evaluate::Trial;
use sapiens_exp::tools::scenario_0;
use sapiens_exp::traces::TraceObserver;
use sapiens_exp::{setup, Config};
use tracing::info;
use tracing_subscriber::EnvFilter;

// FUTURE(ssoudan) BO

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
    #[arg(short, long, default_value = "Make me a cereal bowl with milk")]
    task: String,

    /// Experiments folder
    #[arg(long, default_value = "experiments")]
    experiments_folder: String,
}

impl From<&Args> for Config {
    fn from(args: &Args) -> Self {
        Self {
            model: args.model.clone(),
            max_steps: args.max_steps,
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

    info!("Starting sapiens_cli");

    let toolbox = setup::toolbox().await;

    // prepare scenario 0
    let (toolbox, shared_state) = scenario_0::build(toolbox).await;

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

    let trace_observer = TraceObserver::new();
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

    let trace = { trace_observer.lock().await.trace() };

    // Collect tool utilization stats
    let tool_stats = toolbox.stats().await;

    // Has the task been completed?
    let reached_accepting_state = {
        let guard = shared_state.lock().await;
        guard.has_reached_accepting_state()
    };

    // Build trial
    let trial = Trial::build(config, task, trace, tool_stats, reached_accepting_state);

    // TODO(ssoudan) save trial to file
    // Save to {experiments_folder}/{task_hash}/{config_hash}/{trial_hash}_{nonce}.
    // json

    println!("Trial: {:#?}", trial);

    Ok(())
}
