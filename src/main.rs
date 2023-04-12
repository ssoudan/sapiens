//! Main for botrs
use botrs::{something_with_bash, something_with_rooms};
use dotenvy::dotenv;
use huelib::bridge;

#[tokio::main]
async fn main() {
    // load environment variables from .env file
    dotenv().expect(".env file not found");

    // Discover bridges in the local network and save the first IP address as
    // `bridge_ip`.
    let bridge_ip = bridge::discover_nupnp().unwrap().pop().unwrap();
    println!("Discovered bridge at IP address: {}", bridge_ip);

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

    match bridge.get_all_lights() {
        Ok(lights) => {
            for light in lights {
                println!("{:?}", light);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!("====================");

    match bridge.get_all_groups() {
        Ok(groups) => {
            for group in groups {
                println!("{:?}", group);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // something_with_bash().await;
    something_with_rooms(bridge).await;

    println!("Hello, world!");
}
