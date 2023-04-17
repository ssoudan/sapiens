//! Main for botrs
use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;

use botrs::tools::conclude::ConcludeTool;
use botrs::tools::hue::room::RoomTool;
use botrs::tools::hue::status::StatusTool;
use botrs::tools::python::PythonTool;
use botrs::tools::Toolbox;
use botrs::{something, Config};
use clap::Parser;
use dotenvy::dotenv_override;
use huelib::bridge;

// TODO(ssoudan) split in multiple modules
// TODO(ssoudan) macro
// TODO(ssoudan) better errors for python code
// TODO(ssoudan) tool descriptions: in and out, types, desc, etc.
// TODO(ssoudan) conditional loading of tools
// TODO(ssoudan) More tools: search, wiki, wx, arxiv, etc.
// TODO(ssoudan) Crontab
// TODO(ssoudan) Discord bot
// TODO(ssoudan) logging
// TODO(ssoudan) https://pyo3.rs/v0.17.3/conversions/traits for Input and Output structs?
// TODO(ssoudan) tool to negotiate with the user

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
                bridge::register_user(bridge_ip, "botrs").expect("Failed to register user");
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
    toolbox.add_tool(StatusTool::new(bridge));
    toolbox.add_terminal_tool(ConcludeTool::default());
    toolbox.add_advanced_tool(PythonTool::default());

    // Sanitation
    // remove environment variables that could be used to access the host
    for (k, _) in std::env::vars() {
        std::env::remove_var(&k);
    }

    // let task = "List all the lights in the room with the most lights.";
    // let task = "List all the lights in the room with the least lights.";
    // let task = "How many lights are in each room?";
    // let task = "What are the names of the rooms?";
    // let task = "Sort in ascending order: [2, 3, 1, 4, 5]";
    // let task = "What is the status of the lights in the Office?";
    // let task = "What is the status of the lights where someone is most likely
    // work?";
    // let task = "What are the colors of a rainbow?";
    let task = args.task.clone();
    something(toolbox, openai_client, args.into(), task).await;
}
