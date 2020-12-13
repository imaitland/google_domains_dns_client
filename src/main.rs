extern crate reqwest;
extern crate tokio;
extern crate toml;
extern crate serde;

use std::env;
use std::fs;
use serde::{Deserialize};
use reqwest::{header};

#[derive(Deserialize)]
struct Config {
    username: String,
    password: String,
    hostname: String,
}

#[tokio::main]
async fn get_ip() -> Result<String, reqwest::Error> {
    let ip = reqwest::get("https://api.ipify.org")
        .await?
        .text()
        .await?;
    return Ok(ip)
}

#[tokio::main]
async fn get_google_domains_update(endpoint: String)  -> Result<String, reqwest::Error> {
    let url = reqwest::Url::parse(&endpoint).unwrap();

    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    );

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let res = client
        .get(url)
        .send()
        .await?
        .text()
        .await?;
    return Ok(res);

}

fn main() {
    // Read 'config_path' from the environment variable 'CONFIG'.
    // If 'CONFIG' isn't set, fall back to a default config path.
    let config_path = env::var("CONFIG")
        .unwrap_or(".env.toml".to_string());
    
    let config= fs::read_to_string(config_path);
    let config = match config {
        Ok(contents) => contents,
        Err(_e) => {
            println!("[ ERROR ] Could not find a .env file.");
            return
        },
    };

    let credentials: Config = toml::from_str(&config).unwrap();

    let ip = get_ip();
    let ip = match ip {
        Ok(ip) => ip,
        Err(_e) => {
            println!("[ ERROR ] Could not discover ip.");
            return
        }
    };

    println!("IP: {}", ip);
    // If ip === last updated ip, return.

    let payload = format!("https://{}:{}@domains.google.com/nic/update?hostname={}&myip={}", credentials.username, credentials.password, credentials.hostname, ip);

    let update = get_google_domains_update(payload);
    let update = match update {
        Ok(update) => update, // After a successful update save the just updated ip and return
        Err(_e) => {
            println!("[ ERROR ] Could not connect to Google.");
            return
        }
    };
    println!("Google Says: {}", update);
    // If that worked, log the change in IP address.

}
