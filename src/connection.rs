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

use crate::agent_state::{self, SensorState, WebhookInfo};

pub struct Session {
    pub ws_stream: WebSocketStream<
        async_tungstenite::stream::Stream<
            TokioAdapter<tokio::net::TcpStream>,
            TokioAdapter<tokio_native_tls::TlsStream<tokio::net::TcpStream>>,
        >,
    >,
    hass_protocol: String,
    hass_address: String,
    hass_token: String,
    webhook_url: String,
}

#[derive(Serialize, Deserialize)]
struct RegisterDeviceMessage<T> {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(flatten)]
    payload: T,
}

#[derive(Serialize, Deserialize)]
struct SensorMessage<T> {
    #[serde(rename = "type")]
    message_type: String,
    data: T,
}

impl Session {
    fn calculate_webhook_url(&mut self, webhook_info: &WebhookInfo) -> String {
        if let Some(cloudhook_url) = &webhook_info.cloudhook_url {
            cloudhook_url.to_string()
        } else if let Some(webhook_id) = &webhook_info.webhook_id {
            if let Some(remote_ui_url) = &webhook_info.remote_ui_url {
                format!("{}://{}/api/webhook/{}", self.hass_protocol, remote_ui_url, webhook_id)
            } else {
                format!(
                    "{}://{}/api/webhook/{}",
                    self.hass_protocol, self.hass_address, webhook_id
                )
            }
        } else {
            "".to_string()
        }
    }

    pub fn update_webhook_url(&mut self, webhook_info: &WebhookInfo) {
        self.webhook_url = self.calculate_webhook_url(webhook_info);
    }

    pub async fn connect(hass_protocol: &str, hass_address: &str, hass_token: &str) -> Result<Self, Error> {
        println!("Home-assistant URL: {}", hass_address);
        let url = Url::parse(format!("wss://{}/api/websocket", hass_address).as_str())?;

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
            println!("Authenticated with {}", hass_address);

            Ok(Self {
                ws_stream,
                hass_protocol: hass_protocol.to_string(),
                hass_address: hass_address.to_string(),
                hass_token: hass_token.to_string(),
                webhook_url: "".to_string(),
            })
        } else {
            Err(anyhow!("Authentication failed"))
        }
    }

    pub async fn read_incoming(&mut self) -> Result<(), Error> {
        while let Some(msg) = self.ws_stream.next().await {
            let msg = msg?;
            match msg {
                Message::Ping(_) => {
                    //respond to ping with pong
                    self.ws_stream.send(Message::Pong(vec![])).await?;
                }
                Message::Text(s) => {
                    println!("Received message: {:?}", s);
                    // Continue processing text message... in the future
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn update_registration(&mut self, state: &mut agent_state::State) -> Result<(), Error> {
        self.register_device(state, false).await?;
        self.update_webhook_url(&state.webhook_info);
        Ok(())
    }

    pub async fn register(&mut self, state: &mut agent_state::State) -> Result<(), Error> {
        self.register_device(state, true).await?;
        self.update_webhook_url(&state.webhook_info);
        self.register_sensors(state).await?;
        Ok(())
    }

    async fn register_device(&mut self, state: &mut agent_state::State, new_registration: bool) -> Result<(), Error> {
        let registration_json = json!(RegisterDeviceMessage {
            message_type: if new_registration {
                "register".to_string()
            } else {
                "update_registration".to_string()
            },
            payload: &state.device
        });

        println!("json: {}", registration_json);
        //use reqwest to register device with message
        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "{}://{}/api/mobile_app/registrations",
                self.hass_protocol, self.hass_address
            ))
            .header("Authorization", format!("Bearer {}", self.hass_token))
            .body(registration_json.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            state.webhook_info = response.json().await.expect("json");
            state.registered = true;
            println!("Registered device with Home Assistant");
            Ok(())
        } else {
            println!("Failed to register device with Home Assistant");
            Err(anyhow!("Authentication failed"))
        }
    }

    async fn register_sensors(&mut self, state: &mut agent_state::State) -> Result<(), Error> {
        let client = reqwest::Client::new();
        for sensor in &state.sensors {
            let registration_json = json!(SensorMessage {
                message_type: "register_sensor".to_string(),
                data: sensor.clone()
            });
            println!("json: {}", registration_json);
            let response = client
                .post(&self.webhook_url)
                .body(registration_json.to_string())
                .send()
                .await?;

            if response.status().is_success() {
                println!("Registered sensor {}", sensor.state.unique_id);
                println!(
                    "Registered sensor {:?}",
                    response.text().await.expect("Sensor registration response")
                );
            } else {
                println!("Failed to register device with Home Assistant");
            }
        }
        Ok(())
    }

    pub async fn update_sensor(&mut self, sensors: Vec<SensorState>) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let registration_json = json!(SensorMessage {
            message_type: "update_sensor_states".to_string(),
            data: sensors
        });
        println!("json: {}", registration_json);
        let response = client
            .post(&self.webhook_url)
            .body(registration_json.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            println!(
                "Updated sensors {}",
                response.text().await.expect("Sensor update response")
            );
        } else {
            println!("Failed to update sensors {}", response.status());
        }
        Ok(())
    }
}
