// 1. connect (register) -- https://developers.home-assistant.io/docs/api/native-app-integration/setup
// 2. register sensor -- https://developers.home-assistant.io/docs/api/native-app-integration/sensors
// 3. check webcam status & update sensor
// 4. profit, goto 3
mod agent_state;
mod connection;
mod monitor;
mod config;

use tokio::sync::watch;
use tokio::select;

use agent_state::State;
use connection::Session;
use monitor::microphone;
use monitor::webcam;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = config::load_config();
    let (webcam_state_tx, mut webcam_state_rx) = watch::channel::<bool>(false);
    let (microphone_state_tx, mut microphone_state_rx) = watch::channel::<bool>(false);

    let mut session = Session::connect(&config).await?;
    let mut state = State::init(&config.state_file);
    if !state.registered {
        println!("Registering device with {}", &config.hass_url);
        session.register(&mut state).await?;
        state.save_state(&config.state_file)?;
    } else {
        println!("Device already registered with {}", &config.hass_url);
        if state.device.app_version != env!("CARGO_PKG_VERSION") {
            println!("Version mismatch - updating device with {}", &config.hass_url);
            state = State::new();
            session.update_registration(&mut state).await?;
            state.save_state(&config.state_file)?;
        }
        session.update_webhook_url(&state.webhook_info);
    }

    tokio::spawn(webcam::start(webcam_state_tx));
    tokio::spawn(microphone::start(microphone_state_tx));

    //initial sensor update
    let mut webcam_sensor = state.get_sensor_by_unique_id("webcam").unwrap();
    if webcam::is_webcam_in_use() {
        webcam_sensor.state.value = true;
    }
    session.update_sensor(vec![webcam_sensor.state.clone()]).await?;

    let mut microphone_sensor = state.get_sensor_by_unique_id("microphone").unwrap();
    if microphone::is_microphone_in_use() {
        microphone_sensor.state.value = true;
    }
    session.update_sensor(vec![microphone_sensor.state.clone()]).await?;

    println!("All good! Monitoring...");
    loop {
        select! {
            // The unwrap() here will only panic if all senders have been dropped. This will
            // not happen in normal operation.
            _ = webcam_state_rx.changed() => {
                let state = *webcam_state_rx.borrow();
                if state != webcam_sensor.state.value {
                    webcam_sensor.state.value = state;
                    session.update_sensor(vec![webcam_sensor.state.clone()]).await.unwrap();
                }
            },
            _ = microphone_state_rx.changed() => {
                let state = *microphone_state_rx.borrow();
                if state != microphone_sensor.state.value {
                    microphone_sensor.state.value = state;
                    session.update_sensor(vec![microphone_sensor.state.clone()]).await.unwrap();
                }
            },
            result = session.read_incoming() => {
                // Handle WebSocket message here
                if result.is_err() {
                    println!("Error reading incoming message: {:?}", result.err());
                    continue;
                }
            },
            else => continue,
        }
    }
}
