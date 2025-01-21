#[cfg(target_os = "linux")]
use uinput::{device::{Builder, Device},Event , event};
use std::error::Error;

#[cfg(target_os = "macos")]
use core_foundation;

#[cfg(target_os = "windows")]
use windows::Win32::Devices::HumanInterfaceDevice::*;


#[cfg(target_os = "linux")]
pub fn create_virtual_keyboard() -> Result<Device, Box<dyn Error>> {
    Builder::open("/dev/uinput")
        .map_err(|e| format!("Failed to open /dev/uinput: {}", e))?
        .name("Yolk-Keyboard")?
        .event(Event::Keyboard(event::Keyboard::All))?
        .create()
        .map_err(|e| e.into())
}

#[cfg(target_os = "windows")]
pub fn create_virtual_keyboard() {
    // Windows-specific HID device creation
    println!("Virtual device creation on Windows");
}

#[cfg(target_os = "macos")]
pub fn create_virtual_keyboard() {  // Fixed function name to match others
    // macOS-specific HID device creation
    println!("Virtual device creation on macOS");
}