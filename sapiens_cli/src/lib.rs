//! Sapiens CLI library
#[cfg(feature = "hue")]
use std::net::IpAddr;
#[cfg(feature = "hue")]
use std::rc::Rc;
#[cfg(feature = "hue")]
use std::str::FromStr;

#[cfg(feature = "hue")]
use huelib2::bridge;
use sapiens::tools::Toolbox;
use sapiens_tools::conclude::ConcludeTool;
#[cfg(feature = "hue")]
use sapiens_tools::hue::room::RoomTool;
#[cfg(feature = "hue")]
use sapiens_tools::hue::status::{SetStatusTool, StatusTool};
use sapiens_tools::python::PythonTool;

/// Assemble the toolbox of tools.
///
/// - Uses features to enable/disable tools.
/// - Gets API keys from environment variables.
pub fn assemble_toolbox() -> Toolbox {
    let mut toolbox = Toolbox::default();

    #[cfg(feature = "hue")]
    {
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

        let bridge = bridge::Bridge::new(bridge_ip, username);
        let bridge = Rc::new(bridge);

        toolbox.add_tool(RoomTool::new(bridge.clone()));
        toolbox.add_tool(SetStatusTool::new(bridge.clone()));
        toolbox.add_tool(StatusTool::new(bridge));
    }

    toolbox.add_terminal_tool(ConcludeTool::default());
    toolbox.add_advanced_tool(PythonTool::default());
    toolbox
}
