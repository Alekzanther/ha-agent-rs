use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Error;
use sys_info::{os_type, os_release, hostname};
use users::{get_current_uid, get_user_by_uid};

#[derive(Serialize, Deserialize, Debug)]
pub struct Device { 
    pub device_id: String,
    pub app_id: String,
    pub app_name: String,
    pub app_version: String,
    pub device_name: String,
    pub manufacturer: String,
    pub model: String,
    pub os_name: String,
    pub os_version: String,
    pub supports_encryption: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketInfo {
    pub cloudhook_url: String,
    pub remote_ui_url: String,
    pub secret: String,
    pub webhook_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub registered: bool,
    pub device: Device,
    pub websocket_info: WebSocketInfo
}

impl Metadata {
    //init AgentMetadata
    pub fn init(state_path: &str) -> Self { 
        //if state_path is empty, return default values
        match Self::load_state(state_path) {
            Ok(state) => state,
            Err(_) => {

                let os = os_type().unwrap_or_else(|_| String::from("Unknown OS"));
                let os_version = os_release().unwrap_or_else(|_| String::from("Unknown OS version"));
                let hostname = hostname().unwrap_or_else(|_| String::from("Unknown Hostname"));
                let logged_in_user = get_user_by_uid(get_current_uid()).unwrap();
                let username = logged_in_user.name().to_str().unwrap();

                let device_id = format!("{}@{}", username, hostname);

                Self {
                    registered: false,
                    device: Device {
                        device_id,
                        app_id: "ha-agent-rs".to_string(),
                        app_name: "Home Assistant Agent".to_string(),
                        app_version: "0.1".to_string(),
                        device_name: hostname,
                        manufacturer: "Computer".to_string(),
                        model: "Computer".to_string(),
                        os_name: os.to_string(),
                        os_version: os_version.to_string(),
                        supports_encryption: false,
                    },
                    websocket_info: WebSocketInfo { 
                        cloudhook_url: "".to_string(),
                        remote_ui_url: "".to_string(),
                        secret: "".to_string(),
                        webhook_id: "".to_string(),
                    },
                }
            },
        }
    }

    pub fn save_state(state: Self, path: &str) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(&state)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_state(path: &str) -> Result<Self, Error> {
        let json = fs::read_to_string(path)?;
        let state: Metadata = serde_json::from_str(&json)?;
        Ok(state)
    }
}
