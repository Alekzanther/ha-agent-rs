use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Error;
use sys_info::{os_type, os_release, hostname};
use users::{get_current_uid, get_user_by_uid};

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    // {
    //   "device_id": "ABCDEFGH",
    //   "app_id": "awesome_home",
    //   "app_name": "Awesome Home",
    //   "app_version": "1.2.0",
    //   "device_name": "Robbies iPhone",
    //   "manufacturer": "Apple, Inc.",
    //   "model": "iPhone X",
    //   "os_name": "iOS",
    //   "os_version": "iOS 10.12",
    //   "supports_encryption": true,
    //   "app_data": {
    //     "push_notification_key": "abcdef"
    //   }
    // }
    device_id: String,
    app_id: String,
    app_name: String,
    app_version: String,
    device_name: String,
    manufacturer: String,
    model: String,
    os_name: String,
    os_version: String,
    supports_encryption: bool,
    app_data: String,
}

impl Metadata {
    //init AgentMetadata
    fn init(state_path: &str) -> Self { 
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
                    device_id,
                    app_id: "ha-agent-rs".to_string(),
                    app_name: "Home Assistant Agent".to_string(),
                    app_version: "0.1".to_string(),
                    device_name: hostname,
                    manufacturer: "Alekzanther".to_string(),
                    model: "Computer".to_string(),
                    os_name: os.to_string(),
                    os_version: os_version.to_string(),
                    supports_encryption: false,
                    app_data: "".to_string(),
                }
            },
        }
    }

    fn save_state(state: Self, path: &str) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(&state)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn load_state(path: &str) -> Result<Self, Error> {
        let json = fs::read_to_string(path)?;
        let state: Metadata = serde_json::from_str(&json)?;
        Ok(state)
    }
}
