//! Main for sapiens_cli
use std::sync::Arc;

use clap::Parser;
use colored::Colorize;
use dotenvy::dotenv_override;
use sapiens::chains::Message;
use sapiens::context::{ChatEntry, ChatEntryFormatter, ContextDump, MessageFormatter};
use sapiens::models::{Role, SupportedModel};
use sapiens::{
    models, run_to_the_end, wrap_observer, ChainType, InvocationResultNotification,
    ModelNotification, RuntimeObserver, SapiensConfig,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

// Usability:
// FUTURE(ssoudan) Richer interaction
// FUTURE(ssoudan) More tools: wx, negotiate
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
    #[arg(short, long, default_value = "Tell me a joke.")]
    task: String,

    /// Show the warmup prompt
    #[arg(long)]
    show_warmup_prompt: bool,

    /// Temperature for the model sampling
    /// min: 0, max: 2
    /// The higher the temperature, the crazier the text.
    #[arg(long, default_value_t = 0.)]
    temperature: f32,
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
            Role::Function => msg.red().to_string(),
            Role::Tool => msg.magenta().to_string(),
        }
    }
}

impl MessageFormatter for ColorFormatter {
    fn format(&self, msg: &Message) -> String {
        msg.to_string().yellow().to_string()
    }
}

#[derive(Debug)]
struct Observer {
    /// Whether to show the warm-up prompt
    pub show_warmup_prompt: bool,
}

#[async_trait::async_trait]
impl RuntimeObserver for Observer {
    async fn on_start(&mut self, chat_history: ContextDump) {
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
            let msg = MessageFormatter::format(&ColorFormatter, last_msg);
            println!("{}", msg);
            println!("=============");
        }
    }

    async fn on_model_update(&mut self, event: ModelNotification) {
        let msg = ChatEntryFormatter::format(&ColorFormatter, &event.chat_entry);
        println!("{}", msg);
        println!("=============");
    }

    async fn on_invocation_result(&mut self, event: InvocationResultNotification) {
        match event {
            InvocationResultNotification::InvocationSuccess(i) => {
                println!("{}", i.result.green());
            }
            InvocationResultNotification::InvocationFailure(i) => {
                println!("{}", i.extracted_input.magenta());
                println!("{}", i.e.to_string().red());
            }
            InvocationResultNotification::InvalidInvocation(i) => {
                println!("{}", i.e.to_string().yellow());
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

    let model = match args.model {
        SupportedModel::ChatBison001 => {
            let google_api_key =
                std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY is not set");

            models::vertex_ai::build(google_api_key, Some(args.temperature))
                .await
                .expect("Failed to build model")
        }
        _ => {
            let api_key = std::env::var("OPENAI_API_KEY").ok();
            let api_base = std::env::var("OPENAI_API_BASE").ok();

            models::openai::build(
                args.model.clone(),
                api_key,
                api_base,
                Some(args.temperature),
            )
            .await
            .expect("Failed to build model")
        }
    };

    let task = args.task.clone();
    let config = SapiensConfig {
        model,
        chain_type: args.chain,
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

    let termination_messages = run_to_the_end(config, toolbox, task, w_observer).await;

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
