use crate::config::{HidUuid, DEVICE_NAME};
use btleplug::api::{Central, Peripheral, ScanFilter};
use std::{
    collections::HashSet,
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use uinput::{
    device::Device,
    event::keyboard::{Key, Keyboard, Numeric},
};

fn convert_modifiers(modifiers: u8) -> Vec<Key> {
    let mut keys = Vec::with_capacity(8);

    if modifiers & 0x01 != 0 {
        keys.push(Key::LeftControl);
    }
    if modifiers & 0x02 != 0 {
        keys.push(Key::LeftShift);
    }
    if modifiers & 0x04 != 0 {
        keys.push(Key::LeftAlt);
    }
    if modifiers & 0x08 != 0 {
        keys.push(Key::LeftMeta);
    }
    if modifiers & 0x10 != 0 {
        keys.push(Key::RightControl);
    }
    if modifiers & 0x20 != 0 {
        keys.push(Key::RightShift);
    }
    if modifiers & 0x40 != 0 {
        keys.push(Key::RightAlt);
    }
    if modifiers & 0x80 != 0 {
        keys.push(Key::RightMeta);
    }

    keys
}

fn convert_hid_to_key(code: u8) -> Option<Keyboard> {
    match code {
        // Letters A-Z (0x04-0x1D)
        0x04..=0x1D => Some(Keyboard::Key(match code {
            0x04 => Key::A,
            0x05 => Key::B,
            0x06 => Key::C,
            0x07 => Key::D,
            0x08 => Key::E,
            0x09 => Key::F,
            0x0A => Key::G,
            0x0B => Key::H,
            0x0C => Key::I,
            0x0D => Key::J,
            0x0E => Key::K,
            0x0F => Key::L,
            0x10 => Key::M,
            0x11 => Key::N,
            0x12 => Key::O,
            0x13 => Key::P,
            0x14 => Key::Q,
            0x15 => Key::R,
            0x16 => Key::S,
            0x17 => Key::T,
            0x18 => Key::U,
            0x19 => Key::V,
            0x1A => Key::W,
            0x1B => Key::X,
            0x1C => Key::Y,
            0x1D => Key::Z,
            _ => unreachable!(),
        })),

        // Numbers 1-9,0 (0x1E-0x27)
        0x1E..=0x27 => Some(Keyboard::Key(match code {
            0x1E => Key::_1,
            0x1F => Key::_2,
            0x20 => Key::_3,
            0x21 => Key::_4,
            0x22 => Key::_5,
            0x23 => Key::_6,
            0x24 => Key::_7,
            0x25 => Key::_8,
            0x26 => Key::_9,
            0x27 => Key::_0,
            _ => unreachable!(),
        })),

        // Function keys F1-F12 (0x3A-0x45)
        0x3A..=0x45 => Some(Keyboard::Key(match code {
            0x3A => Key::F1,
            0x3B => Key::F2,
            0x3C => Key::F3,
            0x3D => Key::F4,
            0x3E => Key::F5,
            0x3F => Key::F6,
            0x40 => Key::F7,
            0x41 => Key::F8,
            0x42 => Key::F9,
            0x43 => Key::F10,
            0x44 => Key::F11,
            0x45 => Key::F12,
            _ => unreachable!(),
        })),

        // Arrow keys
        0x4F => Some(Keyboard::Key(Key::Right)),
        0x50 => Some(Keyboard::Key(Key::Left)),
        0x51 => Some(Keyboard::Key(Key::Down)),
        0x52 => Some(Keyboard::Key(Key::Up)),

        // Special characters and control keys
        0x28 => Some(Keyboard::Key(Key::Enter)),
        0x29 => Some(Keyboard::Key(Key::Esc)),
        0x2A => Some(Keyboard::Key(Key::BackSpace)),
        0x2B => Some(Keyboard::Key(Key::Tab)),
        0x2C => Some(Keyboard::Key(Key::Space)),
        0x2D => Some(Keyboard::Key(Key::Minus)),
        0x2E => Some(Keyboard::Key(Key::Equal)),
        0x2F => Some(Keyboard::Key(Key::LeftBrace)), // [
        0x30 => Some(Keyboard::Key(Key::RightBrace)), // ]
        0x32 => Some(Keyboard::Numeric(Numeric::Pound)),
        0x33 => Some(Keyboard::Key(Key::SemiColon)),
        0x34 => Some(Keyboard::Key(Key::Apostrophe)),
        0x35 => Some(Keyboard::Key(Key::Grave)), // `
        0x36 => Some(Keyboard::Key(Key::Comma)),
        0x37 => Some(Keyboard::Key(Key::Dot)),
        0x38 => Some(Keyboard::Key(Key::Slash)),
        0x39 => Some(Keyboard::Key(Key::CapsLock)),
        0x4C => Some(Keyboard::Key(Key::Delete)),
        0x64 => Some(Keyboard::Key(Key::BackSlash)),
        _ => None,
    }
}

pub async fn connect_to_device(central: &impl Central) -> Result<impl Peripheral, Box<dyn Error>> {
    loop {
        println!("scanning for Yolk Keyboard...");

        central.start_scan(ScanFilter::default()).await?;
        //sleep(Duration::from_millis(22)).await;

        let peripherals = central.peripherals().await?;

        for peripheral in peripherals.iter() {
            if let Some(properties) = peripheral.properties().await.ok() {
                if let Some(name) = properties.unwrap().local_name {
                    if name.contains(DEVICE_NAME) {
                        println!("Found {}", DEVICE_NAME);
                        central.stop_scan().await?;

                        println!("Connecting to {}", DEVICE_NAME);
                        peripheral.connect().await?;

                        return Ok(peripheral.clone());
                    }
                }
            }
        }
        central.stop_scan().await?;
        println!("Yolk Keyboard not found. Retrying in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn relay_data(
    ble_device: Arc<impl Peripheral + Send + Sync + 'static>,
    virtual_device: Device,
) -> Result<(), Box<dyn Error>> {
    let virtual_device = Arc::new(Mutex::new(virtual_device));
    let (disconnect_tx, mut disconnect_rx) = mpsc::channel(1);

    loop {
        // Attempt to handle connection
        let result = handle_connection(
            Arc::clone(&ble_device),
            virtual_device.clone(),
            disconnect_tx.clone(),
        )
        .await;

        match result {
            Ok(_) => {
                // Normal shutdown
                println!("Connection closed normally");
                break;
            }
            Err(e) => {
                eprintln!("Connection error: {}. Attempting to reconnect...", e);

                // Attempt to clean up the connection
                if let Err(e) = ble_device.disconnect().await {
                    eprintln!("Error disconnecting: {}", e);
                }

                // Wait before retry
                sleep(Duration::from_secs(2)).await;

                // Check if shutdown was requested
                if disconnect_rx.try_recv().is_ok() {
                    println!("Shutdown requested during reconnection");
                    break;
                }

                // Continue to retry
                continue;
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    ble_device: Arc<impl Peripheral + Send + Sync + 'static>,
    virtual_device: Arc<Mutex<Device>>,
    disconnect_tx: mpsc::Sender<()>,
) -> Result<(), Box<dyn Error>> {
    let hid_uuids = HidUuid::new();
    let device_connected = Arc::new(AtomicBool::new(true));

    // Discover services
    ble_device
        .discover_services()
        .await
        .map_err(|e| format!("Failed to discover services: {}", e))?;

    let services = ble_device.services();
    let hid_service = services
        .iter()
        .find(|s| s.uuid.to_string() == hid_uuids.yolk_hid_service_uuid)
        .ok_or("HID service not found")?;

    let report_characteristic = hid_service
        .characteristics
        .iter()
        .find(|c| c.uuid.to_string() == hid_uuids.report_uuid)
        .ok_or("Report characteristic not found")?;

    // Subscribe to notifications
    ble_device
        .subscribe(report_characteristic)
        .await
        .map_err(|e| format!("Failed to subscribe to notifications: {}", e))?;

    let mut notification_stream = ble_device
        .notifications()
        .await
        .map_err(|e| format!("Failed to get notification stream: {}", e))?;

    let (tx, mut rx) = mpsc::channel(128);

    // Spawn connection monitor
    let monitor_task = {
        let ble_device = Arc::clone(&ble_device);
        let device_connected = Arc::clone(&device_connected);
        let disconnect_tx = disconnect_tx.clone();

        tokio::spawn(async move {
            while device_connected.load(Ordering::SeqCst) {
                sleep(Duration::from_secs(1)).await;
                if !ble_device.is_connected().await.unwrap_or(false) {
                    println!("Device disconnected!");
                    let _ = disconnect_tx.send(()).await;
                    device_connected.store(false, Ordering::SeqCst);
                    break;
                }
            }
        })
    };

    // Spawn BLE listener
    let ble_task = {
        let device_connected = Arc::clone(&device_connected);

        tokio::spawn(async move {
            while let Some(notification) = futures::StreamExt::next(&mut notification_stream).await
            {
                if let Err(e) = tx.try_send(notification.value) {
                    println!("Failed to send notification: {}", e);
                    device_connected.store(false, Ordering::SeqCst);
                    break;
                }
            }
        })
    };

    // Spawn key processing
    let process_task = tokio::spawn(async move {
        let mut previous_keys: HashSet<Keyboard> = HashSet::with_capacity(6);
        let mut new_keys: HashSet<Keyboard> = HashSet::with_capacity(6);
        let mut last_data: [u8; 10] = [0; 10];
        
        while let Some(data) = rx.recv().await {
            if data == last_data {
                continue;
            }
            last_data.copy_from_slice(&data);
            new_keys.clear();
            
            // Process modifier byte (first byte)
            let modifier_keys = convert_modifiers(data[0]);
            new_keys.extend(modifier_keys.into_iter().map(|k| Keyboard::Key(k)));
            
            // Process only the key bytes (skip modifier bytes)
            for &key_code in &data[2..8] {
                if key_code != 0 {
                    if let Some(keyboard_event) = convert_hid_to_key(key_code) {
                        new_keys.insert(keyboard_event);
                    }
                }
            }
            
            // Lock the device only once per update
            let mut device = virtual_device.lock().await;
            
            // Batch process releases
            for key in previous_keys.difference(&new_keys) {
                match key {
                    Keyboard::Key(k) => if let Err(e) = device.release(k) {
                        eprintln!("Failed to release key: {}", e);
                    },
                    Keyboard::Numeric(n) => if let Err(e) = device.release(n) {
                        eprintln!("Failed to release numeric key: {}", e);
                    },
                    _ => {}
                }
            }
            
            // Batch process presses
            for key in new_keys.difference(&previous_keys) {
                match key {
                    Keyboard::Key(k) => if let Err(e) = device.press(k) {
                        eprintln!("Failed to press key: {}", e);
                    },
                    Keyboard::Numeric(n) => if let Err(e) = device.press(n) {
                        eprintln!("Failed to press numeric key: {}", e);
                    },
                    _ => {}
                }
            }
            
            // Single sync after all changes
            device.synchronize()
                .unwrap_or_else(|e| eprintln!("Sync failed: {}", e));
                
            // Swap sets
            std::mem::swap(&mut previous_keys, &mut new_keys);
        }
    });

    // Wait for tasks
    tokio::select! {
        _ = monitor_task => {
            println!("Monitor task completed");
        },
        _ = ble_task => {
            println!("BLE listener task completed");
            device_connected.store(false, Ordering::SeqCst);
        },
        _ = process_task => {
            println!("Key processing task completed");
        },
    }

    Ok(())
}
