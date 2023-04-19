//! Main for sapiens_cli
use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;

use clap::Parser;
use dotenvy::dotenv_override;
use huelib2::bridge;
use sapiens::tools::Toolbox;
use sapiens::{something, Config};
use sapiens_tools::conclude::ConcludeTool;
use sapiens_tools::hue::room::RoomTool;
use sapiens_tools::hue::status::{SetStatusTool, StatusTool};
use sapiens_tools::python::PythonTool;

// Usability:
// TODO(ssoudan) More tools: search, wiki, wx, arxiv, negotiate
// TODO(ssoudan) Conditional loading of tools
// TODO(ssoudan) Discord bot with long-lived conversations
// TODO(ssoudan) Settings
// TODO(ssoudan) Token budget management
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

    let bridge_ip = match std::env::var("HUE_BRIDGE_IP") {
        Ok(ip) => IpAddr::from_str(&ip).expect("Invalid IP address"),
        Err(_) => {
            println!("HUE_BRIDGE_IP env not set. Trying to discover bridge.");
            let bridge_ip = bridge::discover_nupnp().unwrap().pop().unwrap();
            println!(
                "Discovered bridge at IP address: HUE_BRIDGE_IP={}",
                bridge_ip
            );
            bridge_ip
        }
    };

    let username = match std::env::var("HUE_USERNAME") {
        Ok(username) => username,
        Err(_) => {
            println!("HUE_USERNAME env not set. Trying to register a new user.");

            // Register a new user.
            let username =
                bridge::register_user(bridge_ip, "sapiens").expect("Failed to register user");
            println!(
                "Registered a new user - pass it as env: \nHUE_USERNAME={}",
                username
            );
            username
        }
    };

    let openai_client = async_openai::Client::new().with_api_key(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set in configuration file"),
    );

    let bridge = bridge::Bridge::new(bridge_ip, username);
    let bridge = Rc::new(bridge);

    let mut toolbox = Toolbox::default();

    toolbox.add_tool(RoomTool::new(bridge.clone()));
    toolbox.add_tool(SetStatusTool::new(bridge.clone()));
    toolbox.add_tool(StatusTool::new(bridge));
    toolbox.add_terminal_tool(ConcludeTool::default());
    toolbox.add_advanced_tool(PythonTool::default());

    // Sanitation
    // remove environment variables that could be used to access the host
    for (k, _) in std::env::vars() {
        std::env::remove_var(&k);
    }

    let task = args.task.clone();
    something(toolbox, openai_client, args.into(), task).await;
}
