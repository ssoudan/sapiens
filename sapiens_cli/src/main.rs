//! Main for sapiens_cli
use clap::Parser;
use dotenvy::dotenv_override;
use sapiens::{something, Config};

// Usability:
// TODO(ssoudan) Name of the tool that generated the result
// TODO(ssoudan) More tools: search, wx, arxiv, negotiate
// TODO(ssoudan) Discord bot with long-lived conversations
// TODO(ssoudan) Settings
// TODO(ssoudan) Token budget management and completion termination reason
// TODO(ssoudan) Model parameters
// TODO(ssoudan) allow the bot to share its doubt and ask for help
// TODO(ssoudan) Crontab-like scheduling: get a summary of the news daily at 8am
// TODO(ssoudan) better errors for python code
//
// Deployability:
// TODO(ssoudan) Limit how long a tool can run
// TODO(ssoudan) use insta for some of the tests
// TODO(ssoudan) more tests
// TODO(ssoudan) logging
// TODO(ssoudan) monitoring
//
// Adoption:
// TODO(ssoudan) More documentation and examples
// TODO(ssoudan) A site?
//
// Explore:
// TODO(ssoudan) other models?
// TODO(ssoudan) memory?
// TODO(ssoudan) vector stores?
// TODO(ssoudan) prompt optimization
// TODO(ssoudan) multiple models - critic?
// TODO(ssoudan) multi-stage evaluation
// TODO(ssoudan) log the conversation to build a dataset

/// A bot that can do things - or at least try to.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Model to use
    #[arg(long, default_value = "gpt-3.5-turbo")]
    model: String,

    /// Maximum number of steps to execute
    #[arg(short, long, default_value_t = 10)]
    max_steps: u32,

    /// Task to execute
    #[arg(short, long, default_value = "Tell me a joke.")]
    task: String,

    /// Show the warmup prompt
    #[arg(long)]
    show_warmup_prompt: bool,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            model: args.model,
            max_steps: args.max_steps,
            show_warmup_prompt: args.show_warmup_prompt,
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _ = dotenv_override();

    let toolbox = sapiens_cli::assemble_toolbox();

    let openai_client = async_openai::Client::new().with_api_key(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set in configuration file"),
    );

    // Sanitation
    // remove environment variables that could be used to access the host
    for (k, _) in std::env::vars() {
        std::env::remove_var(&k);
    }

    let task = args.task.clone();
    something(toolbox, openai_client, args.into(), task).await;
}
