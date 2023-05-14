// 1. connect (register) -- https://developers.home-assistant.io/docs/api/native-app-integration/setup
// 2. register sensor -- https://developers.home-assistant.io/docs/api/native-app-integration/sensors
// 3. check webcam status & update sensor
// 4. profit, goto 3

use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::WebSocketStream;
use tokio_native_tls;

use futures::{SinkExt, StreamExt};
use url::Url;

use serde_json::json;

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

impl Session {
    pub async fn connect(hass_url: &str, hass_token: &str) -> Result<Self, anyhow::Error> {
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
        let response = ws_stream.next().await.ok_or("didn't receive anything");
        println!("Response: {:?}", response);

        Ok(Self {
            ws_stream,
            hass_url: hass_url.to_string(),
            hass_token: hass_token.to_string(),
        })
    }

    pub async fn register(&mut self, metadata: agent_state::Metadata) -> Result<(), anyhow::Error> {
        let message = Message::text(
            json!({
                "type": "register",
                "app_id": metadata.device.app_id,
                "app_name": metadata.device.app_name,
                "app_version": metadata.device.app_version,
                "device_name": metadata.device.device_name,
                "manufacturer": metadata.device.manufacturer,
                "model": metadata.device.model,
                "os_name": metadata.device.os_name,
                "os_version": metadata.device.os_version,
                "supports_encryption": metadata.device.supports_encryption,
            })
            .to_string(),
        );
        self.ws_stream.send(message).await?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), anyhow::Error> {
        self.ws_stream.close(None).await?;
        Ok(())
    }
}
