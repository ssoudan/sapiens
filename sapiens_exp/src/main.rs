//! Main for sapiens_exp

use std::sync::Arc;

use clap::Parser;
use colored::Colorize;
use dotenvy::dotenv_override;
use sapiens::context::{ChatEntry, ChatEntryFormatter};
use sapiens::openai::Role;
use sapiens::{run_to_the_end, wrap_observer, Config};
use sapiens_exp::setup;
use sapiens_exp::traces::TraceObserver;
use tracing::info;
use tracing_subscriber::EnvFilter;

// TODO(ssoudan) build tools with various kind of inputs and outputs
// TODO(ssoudan) generate tasks - with acceptance criteria
// TODO(ssoudan) tools measure performance from the traces
// TODO(ssoudan) benchmarking harness
// TODO(ssoudan) feature flags
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
    #[arg(short, long, default_value = "Tell me a joke.")]
    task: String,

    /// Show the warmup prompt
    #[arg(long)]
    show_warmup_prompt: bool,
}

impl From<&Args> for Config {
    fn from(args: &Args) -> Self {
        Self {
            model: args.model.clone(),
            max_steps: args.max_steps,
            ..Default::default()
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

    info!("Starting sapiens_cli");

    let toolbox = setup::toolbox().await;

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
        toolbox,
        openai_client,
        (&args).into(),
        task,
        w_trace_observer,
    )
    .await;

    let trace = { trace_observer.lock().await.trace() };
    // TODO(ssoudan) record config with the trace

    println!("Trace: {:#?}", trace);

    Ok(())
}
