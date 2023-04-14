//! Main for botrs
use std::net::IpAddr;
use std::rc::Rc;
use std::str::FromStr;

use botrs::something_with_rooms;
use dotenvy::dotenv;
use huelib::bridge;

#[tokio::main]
async fn main() {
    // load environment variables from .env file
    dotenv().expect(".env file not found");

    let bridge_ip = match std::env::var("HUE_BRIDGE_IP") {
        Ok(ip) => IpAddr::from_str(&ip).expect("Invalid IP address"),
        Err(_) => {
            println!("HUE_BRIDGE_IP not set in .env file. Trying to discover bridge.");
            let bridge_ip = bridge::discover_nupnp().unwrap().pop().unwrap();
            println!("Discovered bridge at IP address: {}", bridge_ip);
            bridge_ip
        }
    };

    let username = match std::env::var("HUE_USERNAME") {
        Ok(username) => username,
        Err(_) => {
            println!("HUE_USERNAME not set in .env file. Trying to register a new user.");

            // Register a new user.
            let username =
                bridge::register_user(bridge_ip, "botrs").expect("Failed to register user");
            println!(
                "Registered a new user - save it in .env as: \nHUE_USERNAME={}",
                username
            );
            username
        }
    };

    let bridge = bridge::Bridge::new(bridge_ip, username);

    // let task = "List all the lights in the room with the most lights.";
    // let task = "List all the lights in the room with the least lights.";
    // let task = "How many lights are in each room?";
    // let task = "What are the names of the rooms?";
    // let task = "Sort in ascending order: [2, 3, 1, 4, 5]";
    // let task = "What is the status of the lights in the Office?";
    let task = "What is the status of the lights where someone is most likely work?";
    // let task = "What are the colors of a rainbow?";
    something_with_rooms(Rc::from(bridge), task, 10).await;
}
