// 1. connect (register) -- https://developers.home-assistant.io/docs/api/native-app-integration/setup
// 2. register sensor -- https://developers.home-assistant.io/docs/api/native-app-integration/sensors
// 3. check webcam status & update sensor
// 4. profit, goto 3

use futures::{SinkExt, StreamExt};

use async_tungstenite::tokio::connect_async;
use async_tungstenite::tungstenite::protocol::Message;

use url::Url;

#[macro_use]
extern crate dotenv_codegen;
use dotenv::dotenv;

use serde_json::json;
mod watcher;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    let hass_url = dotenv!("HASS_URL");
    let hass_token = dotenv!("HASS_TOKEN");
    println!("Home-assistant URL set to: {}", hass_url);
    let url = Url::parse(format!("wss://{}/api/websocket", hass_url).as_str())?;
    watcher::start().await;

    // Then, use the `tungstenite` library to connect to the WebSocket URL
    let (mut ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect websocket");
    let _auth_req = ws_stream.next().await.ok_or("Connection closed");

    // Send a message to register the new device
    let message = Message::text(
        json!({
            "type": "auth",
            "access_token": hass_token
        })
        .to_string(),
    );
    ws_stream.send(message).await?;

    // Read the response message
    let response = ws_stream.next().await.ok_or("didn't receive anything");
    println!("Response: {:?}", response);
    ws_stream.close(None).await?;

    Ok(())
}
