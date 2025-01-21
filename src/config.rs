//src/config.rs
//Device Names
pub const DEVICE_NAME: &str = "Yolk-Keyboard";

//Bluetooth Service/Characteristic UUIDs
pub struct HidUuid {
    pub yolk_hid_service_uuid: String,
    pub report_uuid: String,
    pub _protocol_mode_uuid: String,
    pub _report_map_uuid: String,
    pub _control_point_uuid: String,
}

impl HidUuid {
    pub fn new() -> Self {
        Self {
            yolk_hid_service_uuid: String::from("000066d3-0000-1000-8000-00805f9b34fb"),
            report_uuid: String::from("00002a4d-0000-1000-8000-00805f9b34fb"),
            _protocol_mode_uuid: String::from("00002a4e-0000-1000-8000-00805f9b34fb"),
            _report_map_uuid: String::from("00002a4b-0000-1000-8000-00805f9b34fb"),
            _control_point_uuid: String::from("00002a4c-0000-1000-8000-00805f9b34fb"),
        }
    }
}