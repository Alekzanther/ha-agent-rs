// 1. connect (register) -- https://developers.home-assistant.io/docs/api/native-app-integration/setup
// 2. register sensor -- https://developers.home-assistant.io/docs/api/native-app-integration/sensors
// 3. check webcam status & update sensor
// 4. profit, goto 3

use anyhow::{anyhow, Error};
use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::WebSocketStream;
use tokio_native_tls;

use futures::{SinkExt, StreamExt};
use url::Url;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use reqwest::Client;

use crate::agent_state;

pub struct Session {
    ws_stream: WebSocketStream<
        async_tungstenite::stream::Stream<
            TokioAdapter<tokio::net::TcpStream>,
            TokioAdapter<tokio_native_tls::TlsStream<tokio::net::TcpStream>>,
        >,
    >,
    hass_url: String,
    hass_token: String,
}

#[derive(Serialize, Deserialize)]
struct HaMessage<T> {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(flatten)]
    payload: T,
}

impl Session {
    pub async fn connect(hass_url: &str, hass_token: &str) -> Result<Self, Error> {
        println!("Home-assistant URL: {}", hass_url);
        let url = Url::parse(format!("wss://{}/api/websocket", hass_url).as_str())?;

        // Then, use the `tungstenite` library to connect to the WebSocket URL
        let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect websocket");
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
        let response_json = ws_stream
            .next()
            .await
            .ok_or("didn't receive anything")
            .expect("Authentication")?;
        let response: Value = serde_json::from_str(response_json.to_string().as_str())?;

        if response["type"] == "auth_ok" {
            println!("Authenticated with {}", hass_url);

            Ok(Self {
                ws_stream,
                hass_url: hass_url.to_string(),
                hass_token: hass_token.to_string(),
            })
        } else {
            Err(anyhow!("Authentication failed"))
        }
    }

    pub async fn register(&mut self, metadata: &mut agent_state::Metadata) -> Result<(), Error> {
        let registration_json = json!(HaMessage {
            message_type: "register".to_string(),
            payload: &metadata.device
        });
        println!("json: {}", registration_json);
        //use reqwest to register device with message
        let client = reqwest::Client::new();
        let response = client
            .post(format!("https://{}/api/mobile_app/registrations", &self.hass_url))
            .header("Authorization", format!("Bearer {}", self.hass_token))
            .body(registration_json.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            metadata.websocket_info = response.json().await.expect("json");
            metadata.registered = true;
            println!("Registered device with Home Assistant");
            Ok(())
        } else {
            println!("Failed to register device with Home Assistant");
            Err(anyhow!("Authentication failed"))
        }
    }

    pub async fn disconnect(&mut self) -> Result<(), Error> {
        self.ws_stream.close(None).await?;
        Ok(())
    }
}
