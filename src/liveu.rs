use crate::{config::Liveu as Config_liveu, error::Error};
use read_input::prelude::*;
use reqwest::{
    header::{ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE},
    StatusCode,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

const APPLICATION_ID: &str = "SlZ3SHqiqtYJRkF0zO";
const LIVEU_API: &str = "https://lu-central.liveu.tv/luc/luc-core-web/rest/v0";

#[derive(Deserialize)]
struct Res {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    response: AuthRes,
}

#[derive(Deserialize, Debug, Clone)]
struct AuthRes {
    access_token: String,
    expires_in: u64,
}

#[derive(Deserialize, Debug)]
pub struct UnitInterfaces {
    pub interfaces: Vec<Interface>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Interface {
    pub connected: bool,
    pub name: String,
    pub downlink_kbps: u32,
    pub uplink_kbps: u32,
    pub enabled: bool,
    pub port: String,
    pub technology: String,
    pub up_signal_quality: u32,
    pub down_signal_quality: u32,
    pub active_sim: String,
    pub is_currently_roaming: bool,
    pub kbps: u32,
    pub signal_quality: u32,
}

#[derive(Deserialize, Debug)]
pub struct Unit {
    pub id: String,
    pub reg_code: String,
}

#[derive(Deserialize, Debug)]
pub struct Inventories {
    pub units: Vec<Unit>,
}

pub struct Liveu {
    access_token: Arc<Mutex<String>>,
    config: Config_liveu,
}

impl Liveu {
    pub async fn authenticate(config: Config_liveu) -> Result<Self, Error> {
        let token = if let Ok(token) = Self::get_access_token(&config).await {
            token
        } else {
            return Err(Error::InvalidCredentials);
        };

        Ok(Liveu {
            access_token: Arc::new(Mutex::new(token)),
            config,
        })
    }

    async fn get_access_token(config: &Config_liveu) -> Result<String, Error> {
        let user_session = Uuid::new_v4();
        let client = reqwest::Client::new();

        let res = client
            .post("https://solo-api.liveu.tv/v1_prod/zendesk/userlogin")
            .basic_auth(&config.email, Some(&config.password))
            .header(ACCEPT, "application/json, text/plain, */*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
            .header(CONTENT_TYPE, "application/json;charset=UTF-8")
            .header(
                "x-user-name",
                format!("{}{}", &config.email, &user_session.to_string()),
            )
            .body(r#"{"return_to":"https://solo.liveu.tv/#/dashboard/units"}"#)
            .send()
            .await?
            .json::<Res>()
            .await?;

        Ok(res.data.response.access_token)
    }

    /// Sends the specified request. Gets a new token if unauthorized.
    pub async fn send_request(&self, url: &str) -> Result<reqwest::Response, Error> {
        let mut res = self.try_send_request(&url).await?;

        if res.status() == 401 {
            {
                let mut token = self.access_token.lock().await;
                *token = Self::get_access_token(&self.config).await?;
            }

            dbg!("Getting new token");
            res = self.try_send_request(&url).await?;
        }

        Ok(res)
    }

    pub async fn try_send_request(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();

        client
            .get(url)
            .header(ACCEPT, "application/json, text/plain, */*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
            .header(
                AUTHORIZATION,
                format!("Bearer {}", { &self.access_token.lock().await }),
            )
            .header("application-id", APPLICATION_ID)
            .send()
            .await
    }

    pub async fn get_inventories(&self) -> Result<Inventories, Error> {
        let res = self
            .send_request(&format!("{}/inventories", LIVEU_API))
            .await?;

        if res.status().is_client_error() {
            return Err(Error::NoInventoriesFound);
        }

        let res_json: Value = res.json().await?;
        Ok(serde_json::from_value::<Inventories>(
            res_json["data"]["inventories"][0].to_owned(),
        )?)
    }

    pub async fn get_unit(&self, boss_id: &str) -> Result<Vec<Interface>, Error> {
        let res = self
            .send_request(&format!(
                "{}/units/{}/status/interfaces",
                LIVEU_API, &boss_id
            ))
            .await?;

        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            StatusCode::NO_CONTENT => Ok(vec![]),
            _ => Err(Error::NoUnitsFound),
        }
    }

    /// Gets the location of the boss_id in the inventories
    pub fn get_boss_id_location(inventories: &Inventories) -> usize {
        let size = inventories.units.len();

        if size == 0 {
            panic!("No units found!");
        }

        if size > 1 {
            println!("Found {} units!\n", size);

            for (pos, unit) in inventories.units.iter().enumerate() {
                println!("({}) {}", pos + 1, unit.reg_code);
            }

            let inp = input()
                .msg("\nPlease enter which one you want to use: ")
                .inside_err(
                    1..=size,
                    format!("Please enter a number between 1 and {}: ", size),
                )
                .err("That does not look like a number. Please try again:")
                .get();

            return inp - 1;
        }

        0
    }

    pub async fn get_unit_custom_names(&self, boss_id: &str) -> Result<Vec<Interface>, Error> {
        Ok(self
            .get_unit(boss_id)
            .await?
            .into_iter()
            .filter(|x| x.connected)
            .map(|mut x| {
                match x.port.as_ref() {
                    "eth0" => {
                        x.port = "Ethernet".to_string();
                    }
                    "wlan0" => {
                        x.port = "WiFi".to_string();
                    }
                    "2" => {
                        x.port = "USB1".to_string();
                    }
                    "3" => {
                        x.port = "USB2".to_string();
                    }
                    _ => {}
                }
                x
            })
            .collect())
    }
}
