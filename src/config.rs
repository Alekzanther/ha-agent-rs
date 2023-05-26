use std::env;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// A BLAZINGLY fast agent for Home Assistant
struct Arguments {
    #[structopt(long="url", short="u")]
    /// The URL of your Home Assistant instance
    pub hass_url: Option<String>,
    #[structopt(long="token", short="t")]
    /// The long-lived access token for your Home Assistant user. Read more here: https://www.home-assistant.io/docs/authentication/#your-account-profile
    pub hass_token: Option<String>,
    #[structopt(long="state-file", short="f")]
    /// The file to store the state of the agent in (default: haars.json)
    pub state_file: Option<String>,
}

pub struct Config {
    pub hass_url: url::Url,
    pub hass_token: String,
    pub state_file: String,
}

pub fn load_config() -> Config {
    dotenv::dotenv().ok();

    let args = Arguments::from_args();

    let hass_url_str = args
        .hass_url
        .or_else(|| env::var("HASS_URL").ok())
        .or_else(|| dotenv::var("HASS_URL").ok()).expect("HASS_TOKEN is required");

    let hass_url = url::Url::parse(&hass_url_str).expect("Failed to parse HASS_URL");

    let hass_token = args
        .hass_token
        .or_else(|| env::var("HASS_TOKEN").ok())
        .or_else(|| dotenv::var("HASS_TOKEN").ok()).expect("HASS_TOKEN is required");


    let state_file = args
        .state_file
        .or_else(|| env::var("HAARS_FILE").ok())
        .or_else(|| dotenv::var("HAARS_FILE").ok())
        .unwrap_or_else(|| "haars.json".to_string());

    Config {
        hass_url,
        hass_token,
        state_file,
    }
}
