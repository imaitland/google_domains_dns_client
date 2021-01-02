/*
 * Many ISPs do not provide free static IP addresses to their customers. This means that servers
 * running at home, might have their IP addresses changed, at which point they would become
 * inaccessible to the DNS service that routes traffic to them from the wider world.
 * To make sure the server remains accessible, we notify certain services, notably the DNS service 
 * when the IP has changed.
 */

extern crate reqwest;
extern crate tokio;
extern crate toml;
extern crate serde;

use std::env;
use std::fs;
use serde::{Deserialize};
use std::io::{Write, BufReader, BufRead};

enum Service {
    GoogleDomainsDNS(GoogleConfig),
}

fn tell_service(ip: &str, service : Service) -> Result<String, reqwest::Error> {
    match service {
        Service::GoogleDomainsDNS(GoogleConfig) => Ok(google_domains_update(GoogleConfig.endpoint(ip))?)
   }
}

#[derive(Deserialize)]
struct GoogleConfig {
    username: String,
    password: String,
    hostname: String,
}

impl GoogleConfig {
    fn endpoint(&self, ip: &str) -> String {
        return format!("https://{}:{}@domains.google.com/nic/update?hostname={}&myip={}", self.username, self.password, self.hostname, ip);
    }
} 

#[derive(Deserialize)]
struct Config {
    google_1: GoogleConfig, // '@' A record
    google_2: GoogleConfig // 'www' A record
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

fn parse_config (path: &String) -> Result<Config, std::io::Error>  {
    let a = fs::read_to_string(path)?;
    let b : Config = toml::from_str(&a)?;
    Ok(b)
}

fn read_last_line(path: &str) -> Result<String, std::io::Error> {

    let input = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let last = BufReader::new(input).lines().last();

    if last.is_none() {
        return Ok(String::from("0.0.0.0"))
    } else {
        let line = last.unwrap();
        match line {
            Ok(line) => return Ok(line),
            Err(_e) => return Ok(String::from("nooo!"))
        }
    }
}

fn append_to_file(path: &str, s: &String) -> Result<(), std::io::Error> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(path)?;
    writeln!(file, "{}", s)?;
    return Ok(())
}

fn main() {
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
    let ip = match &ip {
        Ok(ip) => ip,
        Err(e) => {
            println!("[ ERROR ] Could not discover ip. {}", e);
            return
        }
    };

    static LOG_FILE: &str = "log.txt";

    let old_ip = read_last_line(LOG_FILE);
    let old_ip = match &old_ip {
        Ok(old_ip) => old_ip,
        Err(e) => {
            println!("[ ERROR ] Could not read previous ip from log. {}", e);
            return
        }
    };

    if old_ip == ip {
        println!("[ INFO ] Old ip: {} \n New ip: {} \n No change in ip.", old_ip, ip );
        return
    }

    // curry our service
    let tell_service_ip = move |service| tell_service(&ip[..], service);  
    
    let goog = Service::GoogleDomainsDNS(config.google_1);
    let response = tell_service_ip(goog);
    match response {
        Ok(response) => response,
        Err(e) => {
            println!("[ ERROR ] Could not tell the google service. {}", e);
            return
        }
    };

    let goog = Service::GoogleDomainsDNS(config.google_2);
    let response = tell_service_ip(goog);
    match response {
        Ok(response) => response,
        Err(e) => {
            println!("[ ERROR ] Could not tell the google service. {}", e);
            return
        }
    };

    append_to_file(LOG_FILE, &ip).unwrap();
    return
}
