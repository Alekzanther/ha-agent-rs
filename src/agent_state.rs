use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error;
use sys_info::{hostname, os_release, os_type};
use users::{get_current_uid, get_user_by_uid};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Sensor {
    #[serde(flatten)]
    pub state: SensorState,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SensorState {
    #[serde(rename = "state")]
    pub value: bool,
    pub unique_id: String,
    #[serde(rename = "type")]
    pub sensor_type: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WebhookInfo {
    pub cloudhook_url: Option<String>,
    pub remote_ui_url: Option<String>,
    pub secret: Option<String>,
    pub webhook_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct State {
    pub registered: bool,
    pub device: Device,
    pub webhook_info: WebhookInfo,
    pub sensors: Vec<Sensor>,
}

impl State {
    pub fn new() -> Self {
        let os_name = os_type().unwrap_or_else(|_| String::from("Unknown OS"));
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
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                device_name: hostname,
                manufacturer: "Computer".to_string(),
                model: "Computer".to_string(),
                os_name,
                os_version,
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
    //init AgentMetadata
    pub fn init(state_path: &str) -> Self {
        //if state_path is empty, return default values
        match Self::load_state(state_path) {
            Ok(state) => state,
            Err(_) => Self::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new_state() {
        let state = State::new();

        assert_eq!(state.registered, false);
        assert_eq!(state.device.device_id.contains('@'), true);
        assert_eq!(state.device.app_id, "ha-agent-rs");
        assert_eq!(state.device.app_name, "Home Assistant Agent");
        assert_eq!(state.sensors.len(), 2);
        assert_eq!(state.sensors[0].name, "Webcam");
        assert_eq!(state.sensors[1].name, "Microphone");
    }

    #[test]
    fn test_init_state_with_empty_path() {
        let state = State::init("");

        assert_eq!(state.registered, false);
        assert_eq!(state.device.device_id.contains('@'), true);
        assert_eq!(state.device.app_id, "ha-agent-rs");
        assert_eq!(state.device.app_name, "Home Assistant Agent");
        assert_eq!(state.sensors.len(), 2);
        assert_eq!(state.sensors[0].name, "Webcam");
        assert_eq!(state.sensors[1].name, "Microphone");
    }

    #[test]
    fn test_save_and_load_state() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("state.json");
        let state = State::new();

        state.save_state(file_path.to_str().unwrap()).unwrap();

        let loaded_state = State::load_state(file_path.to_str().unwrap()).unwrap();

        assert_eq!(state, loaded_state);
    }

    #[test]
    fn test_get_sensor_by_unique_id() {
        let state = State::new();

        assert!(state.get_sensor_by_unique_id("webcam").is_some());
        assert!(state.get_sensor_by_unique_id("microphone").is_some());
        assert!(state.get_sensor_by_unique_id("nonexistent").is_none());
    }
}
