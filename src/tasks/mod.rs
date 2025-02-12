mod mqtt;
mod wifi;

use core::ffi::CStr;

pub use mqtt::mqtt_handler;
pub use wifi::{connection, net_task};

// set your broker and client info
const MQTT_SERVERNAME: &str = "your.broker.server";
const SERVERNAME: &CStr = c"your.broker.server";
const MQTT_SERVERPORT: u16 = 8883;
const MQTT_CLIENT_ID: &str = "client-id";
const MQTT_USR_NAME: &str = "";
const MQTT_USR_PASS: [u8; 0] = *b"";
