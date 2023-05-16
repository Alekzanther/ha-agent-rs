// 1. connect (register) -- https://developers.home-assistant.io/docs/api/native-app-integration/setup
// 2. register sensor -- https://developers.home-assistant.io/docs/api/native-app-integration/sensors
// 3. check webcam status & update sensor
// 4. profit, goto 3
mod agent_state;
mod connection;
mod watcher;

use std::sync::mpsc;

#[macro_use]
extern crate dotenv_codegen;
use dotenv::dotenv;

use agent_state::Metadata;
use connection::Session;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    let hass_url = dotenv!("HASS_URL");
    let hass_token = dotenv!("HASS_TOKEN");
    let state_file = dotenv!("HAARS_FILE");
    let (webcam_state_tx, webcam_state_rx) = mpsc::channel::<bool>();
    let _webcam_watcher = tokio::spawn(watcher::start(webcam_state_tx));
    let mut session = Session::connect(hass_url, hass_token).await?;
    let mut state = Metadata::init(state_file);

    if !state.registered {
        println!("Registering device with {}", hass_url);
        session.register(&mut state).await?;
        state.save_state(state_file)?;
    } else {
        println!("Device already registered with {}", hass_url);
    }

    println!("All good! Monitoring...");
    loop {
        let webcam_state = webcam_state_rx.recv().unwrap();
        if webcam_state {
            println!("In use!");
        } else {
            println!("Not used...")
        }
    }
}
