mod bt_core;
mod config;
mod platform;

use btleplug::api::Manager as _;
use platform::create_virtual_keyboard;
use bt_core::connect_to_device;
use bt_core::relay_data;
use tokio::{self, time::{sleep,Duration}};

// Main function showing usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    let manager = btleplug::platform::Manager::new().await?;
    let central = manager.adapters().await?
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapters found")?;

    loop {
        println!("Starting Yolk Keyboard service...");
        
        // Create virtual keyboard
        let device = match create_virtual_keyboard() {
            Ok(device) => device,
            Err(e) => {
                eprintln!("Failed to create virtual keyboard: {}. Retrying in 5 seconds...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // Connect to BLE device
        let peripheral = match connect_to_device(&central).await {
            Ok(peripheral) => peripheral,
            Err(e) => {
                eprintln!("Failed to connect: {}. Retrying...", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        println!("Connected to Yolk-Keyboard");
        // Start relay
        if let Err(e) = relay_data(peripheral.into(), device).await {
            eprintln!("Relay error: {}. Restarting...", e);
            sleep(Duration::from_secs(2)).await;
            continue;
        }
    }
}