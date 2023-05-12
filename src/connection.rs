// 1. connect (register) -- https://developers.home-assistant.io/docs/api/native-app-integration/setup
// 2. register sensor -- https://developers.home-assistant.io/docs/api/native-app-integration/sensors
// 3. check webcam status & update sensor
// 4. profit, goto 3

use std::net::TcpStream;

use async_tungstenite::stream::Stream;
use async_tungstenite::WebSocketStream;

use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::tungstenite::protocol::Message;

use futures::{SinkExt, StreamExt};
use url::Url;

use serde_json::json;

pub struct Session {
    ws_stream: WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
}

impl Session {
    pub async fn connect(hass_url: &str, hass_token: &str) -> Result<Self, anyhow::Error> {
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

        Ok(Self { ws_stream })
    }
    pub async fn disconnect(&self) -> Result<(), anyhow::Error> {
        self.ws_stream.close(None).await?
    }
}
