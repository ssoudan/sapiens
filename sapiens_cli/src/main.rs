//! Main for sapiens_cli
use std::sync::Arc;

use clap::Parser;
use colored::Colorize;
use dotenvy::dotenv_override;
use sapiens::context::{ChatEntry, ChatEntryFormatter, ChatHistoryDump};
use sapiens::models::openai::SupportedModel;
use sapiens::models::Role;
use sapiens::{
    run_to_the_end, wrap_observer, InvocationFailureNotification, InvocationSuccessNotification,
    ModelUpdateNotification, SapiensConfig, StepObserver,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

// Usability:
// FUTURE(ssoudan) Richer interaction
// FUTURE(ssoudan) More tools: wx, negotiate
// FUTURE(ssoudan) Settings
// FUTURE(ssoudan) Token budget management and completion termination reason
// FUTURE(ssoudan) Model parameters
// FUTURE(ssoudan) allow the bot to share its doubt and ask for help
// FUTURE(ssoudan) Crontab-like scheduling: get a summary of the news daily
// FUTURE(ssoudan) better errors for python code
//
// Deployability:
// FUTURE(ssoudan) Limit how long a tool can run
// FUTURE(ssoudan) monitoring
//
// Adoption:
// FUTURE(ssoudan) More documentation and examples
// FUTURE(ssoudan) GH Pages
//
// Explore:
// FUTURE(ssoudan) memory?
// FUTURE(ssoudan) vector stores?
// FUTURE(ssoudan) prompt optimization
// FUTURE(ssoudan) multiple models - critic?
// FUTURE(ssoudan) multi-stage evaluation

/// A bot that can do things - or at least try to.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Model to use
    #[arg(long, default_value_t = SupportedModel::GPT3_5Turbo, value_enum, env)]
    model: SupportedModel,

    /// Maximum number of steps to execute
    #[arg(short, long, default_value_t = 10)]
    max_steps: usize,

    /// Minimum tokens for completion
    #[arg(long, default_value_t = 512)]
    min_tokens_for_completion: usize,

    /// Max tokens for the model to generate
    #[arg(long)]
    max_tokens: Option<usize>,
    /// Task to execute
    #[arg(short, long, default_value = "Tell me a joke.")]
    task: String,

    /// Show the warmup prompt
    #[arg(long)]
    show_warmup_prompt: bool,
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

#[derive(Debug)]
struct Observer {
    /// Whether to show the warm-up prompt
    pub show_warmup_prompt: bool,
}

#[async_trait::async_trait]
impl StepObserver for Observer {
    async fn on_start(&mut self, chat_history: ChatHistoryDump) {
        if self.show_warmup_prompt {
            let formatter = ColorFormatter {};
            let msgs = chat_history.format(&formatter);

            for msg in msgs {
                println!("{}", msg);
                println!("=============");
            }
        } else {
            // Show only the last message
            let last_msg = chat_history.messages.last().unwrap();
            let msg = ColorFormatter.format(last_msg);
            println!("{}", msg);
            println!("=============");
        }
    }

    async fn on_model_update(&mut self, event: ModelUpdateNotification) {
        let msg = ColorFormatter.format(&event.chat_entry);
        println!("{}", msg);
        println!("=============");
    }

    async fn on_invocation_success(&mut self, event: InvocationSuccessNotification) {
        match event.res {
            Ok(tool_output) => {
                let msg = ColorFormatter.format(&tool_output);
                println!("{}", msg);
            }
            Err(e) => {
                println!("{}", e.to_string().red());
            }
        }

        println!("=============");
    }

    async fn on_invocation_failure(&mut self, event: InvocationFailureNotification) {
        match event.res {
            Ok(tool_output) => {
                let msg = tool_output.msg.yellow();
                println!("{}", msg);
            }
            Err(e) => {
                println!("{}", e.to_string().red());
            }
        }

        println!("=============");
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

    let toolbox = sapiens_tools::setup::toolbox_from_env().await;

    let api_key = std::env::var("OPENAI_API_KEY").ok();
    let api_base = std::env::var("OPENAI_API_BASE").ok();

    let task = args.task.clone();
    let config = SapiensConfig {
        model: sapiens::models::openai::build(args.model.clone(), api_key, api_base, Some(0.))
            .await
            .expect("Failed to build model"),
        max_steps: args.max_steps,
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

    let observer = Observer {
        show_warmup_prompt: args.show_warmup_prompt,
    };

    let observer = wrap_observer(observer);

    let w_observer = Arc::downgrade(&observer);

    let termination_messages = run_to_the_end(toolbox, config, task, w_observer).await;

    if let Err(e) = termination_messages {
        println!("{}", e.to_string().red());
        return Ok(());
    }

    let termination_messages = termination_messages.unwrap();

    for message in termination_messages {
        println!(
            "The original question was: {} ",
            message.original_question.green()
        );
        println!("And the conclusion is: {} ", message.conclusion.blue());
    }

    Ok(())
}
