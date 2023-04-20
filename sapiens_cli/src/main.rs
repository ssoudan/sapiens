//! Main for sapiens_cli
use clap::Parser;
use colored::Colorize;
use dotenvy::dotenv_override;
use sapiens::context::{ChatEntry, ChatEntryFormatter, ChatHistory};
use sapiens::{work, Config, Error, Role, TaskProgressUpdateHandler};

// FIXME(ssoudan) - deterministic order of tools in the prompt
//
// Usability:
// NOW(ssoudan) - modularize the main loop
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
// TODO(ssoudan) categorize the outcomes - count the number of steps

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

#[derive(Debug)]
struct Handler {
    /// Whether to show the warm-up prompt
    pub show_warmup_prompt: bool,
}

impl TaskProgressUpdateHandler for Handler {
    fn on_start(&self, chat_history: &ChatHistory) {
        if self.show_warmup_prompt {
            let msgs = chat_history.format(ColorFormatter {});

            for msg in msgs {
                println!("{}", msg);
                println!("=============");
            }
        } else {
            // Show only the last message
            let last_msg = chat_history.iter().last().unwrap();
            let msg = ColorFormatter.format(&last_msg.into());
            println!("{}", msg);
            println!("=============");
        }
    }

    fn on_tool_update(&self, chat_message: ChatEntry, success: bool) {
        if success {
            let msg = ColorFormatter.format(&chat_message);
            println!("{}", msg);
        } else {
            let msg = chat_message.msg.yellow();
            println!("{}", msg);
        }
        println!("=============");
    }

    fn on_tool_error(&self, error: Error) {
        println!("{}", error.to_string().red());
        println!("=============");
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
    assert!(
        std::env::vars().next().is_none(),
        "Environment is not empty"
    );

    let task = args.task.clone();
    let termination_messages = work(
        toolbox,
        openai_client,
        (&args).into(),
        task,
        Handler {
            show_warmup_prompt: args.show_warmup_prompt,
        },
    )
    .await
    .unwrap();

    for message in termination_messages {
        println!(
            "The original question was: {} ",
            message.original_question.green()
        );
        println!("And the conclusion is: {} ", message.conclusion.blue());
    }
}
