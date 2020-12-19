extern crate reqwest;
extern crate tokio;
extern crate toml;
extern crate serde;

use std::env;
use std::fs;
use serde::{Deserialize};
use std::io::{Write, LineWriter, BufReader, BufRead};

// Add services that you'd like to notify when your IP changes here.
enum Service {
    GoogleDomainsDNS(GoogleConfig),
}

// Pattern match on the Service Enum
fn tell_service(ip: String, service : Service) -> Result<String, reqwest::Error> {
    match service {
        Service::GoogleDomainsDNS(GoogleConfig) => Ok(google_domains_update(GoogleConfig.endpoint(ip))?)
   }
}

// Struct for the google domains credentials requirement.
#[derive(Deserialize)]
struct GoogleConfig {
    username: String,
    password: String,
    hostname: String,
}

// Method on the GoogleConfig struct to build the google domains endpoint from the google config credentials.
impl GoogleConfig {
    fn endpoint(&self, ip: String) -> String {
        return format!("https://{}:{}@domains.google.com/nic/update?hostname={}&myip={}", self.username, self.password, self.hostname, ip);
    }
} 

// Struct used to deserialize the .env.toml file. 
#[derive(Deserialize)]
struct Config {
    google: GoogleConfig,
}

// Get IP from the ipify service.
#[tokio::main]
async fn get_ip() -> Result<String, reqwest::Error> {
    let ip = reqwest::get("https://api.ipify.org")
        .await?
        .text()
        .await?;
    return Ok(ip)
}

// Tell Google about IP change.
#[tokio::main]
async fn google_domains_update(endpoint: String)  -> Result<String, reqwest::Error> {
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

// Read config toml.
fn parse_config (path: &String) -> Result<Config, std::io::Error>  {
    let a = fs::read_to_string(path)?;
    let b : Config = toml::from_str(&a)?;
    Ok(b)
}

// Read the last line from the log.
fn read_last_line(path: String) -> Result<String, std::io::Error> {
    let input = fs::File::open(path)?;
    BufReader::new(input).lines().last().unwrap()
}

// Append a line to the log.
fn append_to_file(path: &str, s: String) -> Result<(), std::io::Error> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(path)?;
    writeln!(file, "{}/n", s)
}

fn main() {
    // Read 'config_path' from the environment variable 'CONFIG'.
    // If 'CONFIG' isn't set, fall back to a default config path.
    let config_path = env::var("CONFIG")
        .unwrap_or(".env.toml".to_string());
    

    let config: Result<Config, std::io::Error> = parse_config(&config_path);
    let config: Config= match config {
        Ok(c) => c,
        Err(e) => {
            println!("[ ERROR ] Could not get configuration. {}", e);
            return
        }
    };

    let ip = get_ip();

    let ip = match ip {
        Ok(ip) => ip,
        Err(e) => {
            println!("[ ERROR ] Could not discover ip. {}", e);
            return
        }
    };

    // TODO: If, current ip === last ip saved in log, return as there is no need to tell google.
    /*
     * static LOG_FILE: &str = "log.txt";
     * let old_ip = read_last_line("log.txt")
     * if ip === old_ip {return}
     */

    // curry our service
    let tell_service_ip = move |service| tell_service(ip, service);  

    let goog = Service::GoogleDomainsDNS(config.google);
    let response = tell_service_ip(goog);
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            println!("[ ERROR ] Could not tell the google service. {}", e);
            return
        }
    };

    // TODO: Append new ip to log.
    /*
     * append_to_file(LOG_FILE, ip)
     *
     */

}
