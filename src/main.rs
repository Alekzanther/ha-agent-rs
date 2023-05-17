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

use agent_state::State;
use connection::Session;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    let hass_protocol = dotenv!("HASS_PROTOCOL");
    let hass_address = dotenv!("HASS_ADDRESS");
    let hass_token = dotenv!("HASS_TOKEN");
    let state_file = dotenv!("HAARS_FILE");
    let (webcam_state_tx, webcam_state_rx) = mpsc::channel::<bool>();
    let _webcam_watcher = tokio::spawn(watcher::start(webcam_state_tx));
    let mut session = Session::connect(hass_protocol, hass_address, hass_token).await?;
    let mut state = State::init(state_file);

    if !state.registered {
        println!("Registering device with {}", hass_address);
        session.register(&mut state).await?;
        state.save_state(state_file)?;
    } else {
        session.update_webhook_url(&state.webhook_info);
        println!("Device already registered with {}", hass_address);
    }

    //initial sensor update
    let mut webcam_sensor = state.get_sensor_by_unique_id("webcam").unwrap();
    if watcher::is_webcam_in_use() {
        webcam_sensor.state.value = true;
    }
    session.update_sensor(vec![webcam_sensor.state.clone()]).await?;

    println!("All good! Monitoring...");
    loop {
        let webcam_state = webcam_state_rx.recv().unwrap();
        if webcam_state {
            webcam_sensor.state.value = true;
            session.update_sensor(vec![webcam_sensor.state.clone()]).await?;
        } else {
            webcam_sensor.state.value = false;
            session.update_sensor(vec![webcam_sensor.state.clone()]).await?;
        }
        //read websocket messages... yes here... until I've figured out how to split mpsc/webscoket
        //coms better
        //session.read_incoming().await?;
    }
}
