use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error;
use sys_info::{hostname, os_release, os_type};
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sensor {
    #[serde(flatten)]
    pub state: SensorState,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SensorState {
    #[serde(rename = "state")]
    pub value: bool,
    pub unique_id: String,
    #[serde(rename = "type")]
    pub sensor_type: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookInfo {
    pub cloudhook_url: Option<String>,
    pub remote_ui_url: Option<String>,
    pub secret: Option<String>,
    pub webhook_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub registered: bool,
    pub device: Device,
    pub webhook_info: WebhookInfo,
    pub sensors: Vec<Sensor>,
}

impl State {
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

                let webcam_sensor = Sensor {
                    name: "Webcam".to_string(),
                    state: SensorState {
                        value: false,
                        unique_id: "webcam".to_string(),
                        sensor_type: "binary_sensor".to_string(),
                        icon: "mdi:webcam".to_string(),
                    },
                };
                let microphone_sensor = Sensor {
                    name: "Microphone".to_string(),
                    state: SensorState {
                        value: false,
                        unique_id: "microphone".to_string(),
                        sensor_type: "binary_sensor".to_string(),
                        icon: "mdi:microphone".to_string(),
                    },
                };

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
                    webhook_info: WebhookInfo {
                        cloudhook_url: None,
                        remote_ui_url: None,
                        secret: None,
                        webhook_id: None,
                    },
                    sensors: vec![webcam_sensor, microphone_sensor],
                }
            }
        }
    }

    pub fn save_state(&self, path: &str) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(path, json)?;
        println!("Saved state to {}", path);
        Ok(())
    }

    pub fn load_state(path: &str) -> Result<Self, Error> {
        let json = fs::read_to_string(path)?;
        let state: State = serde_json::from_str(&json)?;
        Ok(state)
    }

    pub fn get_sensor_by_unique_id(&self, unique_id: &str) -> Option<Sensor> {
        for sensor in &self.sensors {
            if sensor.state.unique_id == unique_id {
                return Some(sensor.clone());
            }
        }
        None
    }
}
