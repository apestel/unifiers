use std::collections::HashMap;

use anyhow::{anyhow, Ok, Result};
use clap::{Parser, ValueEnum};
//use clap;
use reqwest::{self};
use thiserror::Error;

use config::Config;

use serde::{self, Deserialize, Serialize};
use serde_json::json;

struct UnifiApi<'a> {
    base_url: &'a str,
    login: &'a str,
    password: &'a str,
    client: reqwest::blocking::Client,
    port_enable_profile_id: String,
    port_disable_profile_id: String,
}

// {
//     "port_overrides": [
//         {
//             "port_idx": 10,
//             "poe_mode": "off",
//             "portconf_id": "6263dec9fadf8300220bd18b",
//             "port_security_mac_address": [],
//             "stp_port_mode": true,
//             "autoneg": true,
//             "port_security_enabled": false
//         }
//     ]
// }⏎
#[derive(Serialize)]
struct PortOverride<'a> {
    port_idx: i32,
    poe_mode: &'a str,
    portconf_id: &'a str,
    port_security_mac_address: Vec<String>,
    stp_port_mode: bool,
    autoneg: bool,
    port_security_enabled: bool,
}

// #[derive(Serialize)]
// #[serde(untagged)]
// enum PoeMode {
//     Auto,
//     Pasv24,
//     Passthrough,
//     Off,
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum UnifiApiReturnCode {
    Ok,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct UnifiApiMetaResponse {
    rc: UnifiApiReturnCode,
    msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct UnifiApiResponse {
    meta: UnifiApiMetaResponse,
}

#[derive(Debug, Error)]
enum UnifiApiErrorCode {
    #[error("login required")]
    LoginRequired,
    #[error("{0:?}")]
    Unknown(String),
}

impl From<String> for UnifiApiErrorCode {
    fn from(value: String) -> Self {
        let value = value.as_str();
        match value {
            "api.err.LoginRequired" => Self::LoginRequired,
            _ => Self::Unknown(value.to_owned()),
        }
    }
}

impl UnifiApi<'_> {
    pub fn new<'a>(base_url: &'a str, login: &'a str, password: &'a str) -> Result<UnifiApi<'a>> {
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .build()?;
        Ok(UnifiApi {
            base_url,
            login,
            password,
            client,
            port_disable_profile_id: "".to_owned(),
            port_enable_profile_id: "".to_owned(),
        })
    }

    pub fn login(&self) -> Result<()> {
        let url = format!("{}/api/login", self.base_url);
        log::debug!("URL: {}", url);
        let json_body = json!({"username": self.login, "password": self.password});
        let response: UnifiApiResponse = self.client.post(url).json(&json_body).send()?.json()?;

        match response.meta.rc {
            UnifiApiReturnCode::Ok => {
                log::info!("Login OK");
                Ok(())
            }
            UnifiApiReturnCode::Error => {
                Err(UnifiApiErrorCode::Unknown(response.meta.msg.unwrap()).into())
            }
        }
    }

    fn change_port_settings(
        &self,
        device_id: &str,
        port_number: i32,
        port_status: &str,
    ) -> Result<()> {
        let url: String = format!("{}/api/s/default/rest/device/{}", self.base_url, device_id);
        let json_body = json!({"port_overrides": [PortOverride{port_idx:port_number,poe_mode:"auto",portconf_id:port_status,port_security_mac_address:vec![],autoneg:true,port_security_enabled:false, stp_port_mode: true }]});
        log::debug!(
            "Request: {}",
            serde_json::to_string_pretty(&json_body).unwrap()
        );
        let response: UnifiApiResponse = self.client.put(url).json(&json_body).send()?.json()?;

        match response.meta.rc {
            UnifiApiReturnCode::Ok => {
                log::info!("Command OK");
                Ok(())
            }
            UnifiApiReturnCode::Error => {
                match UnifiApiErrorCode::from(response.meta.msg.unwrap()) {
                    UnifiApiErrorCode::LoginRequired => {
                        log::info!("Not logged in, trying to connect...");
                        self.login()?;
                        self.change_port_settings(device_id, port_number, port_status)
                    }
                    UnifiApiErrorCode::Unknown(v) => Err(UnifiApiErrorCode::Unknown(v).into()),
                }
            }
        }
    }

    pub fn set_port_enable_profile_id(&mut self, port_enable_profile_id: String) {
        self.port_enable_profile_id = port_enable_profile_id
    }

    pub fn set_port_disable_profile_id(&mut self, port_disable_profile_id: String) {
        self.port_disable_profile_id = port_disable_profile_id
    }

    pub fn disable_port(&self, device_id: &str, port_number: i32) -> Result<()> {
        self.change_port_settings(device_id, port_number, &self.port_disable_profile_id)
    }

    pub fn enable_port(&self, device_id: &str, port_number: i32) -> Result<()> {
        self.change_port_settings(device_id, port_number, &self.port_enable_profile_id)
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum PortProfile {
    Up,
    Down,
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_file_path: String,
    /// Port number to change profile
    #[arg(short, long)]
    port_number: i32,

    #[arg(value_enum)]
    profile: PortProfile,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let settings = Config::builder()
        .add_source(config::File::with_name(&args.config_file_path))
        .build()?;
    let settings = settings.try_deserialize::<HashMap<String, String>>()?;
    let base_url = settings
        .get("base_url")
        .ok_or(anyhow!("can't find base_url Unifi API setting"))?;
    let login = settings
        .get("login")
        .ok_or(anyhow!("can't find login Unifi API setting"))?;
    let password = settings
        .get("password")
        .ok_or(anyhow!("can't find password Unifi API setting"))?;
    let device_id = settings
        .get("device_id")
        .ok_or(anyhow!("can't find device_id setting"))?;
    let port_profile_down = settings
        .get("port_profile_down")
        .ok_or(anyhow!("can't find device_id setting"))?;
    let port_profile_up: &String = settings
        .get("port_profile_up")
        .ok_or(anyhow!("can't find device_id setting"))?;

    let mut api = UnifiApi::new(base_url, login, password)?;
    api.set_port_disable_profile_id(port_profile_down.to_string());
    api.set_port_enable_profile_id(port_profile_up.to_string());

    if args.port_number > 0 {
        return match args.profile {
            PortProfile::Up => api.enable_port(device_id, args.port_number),
            PortProfile::Down => api.disable_port(device_id, args.port_number),
        };
    }
    Ok(())
}
