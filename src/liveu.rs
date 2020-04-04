//use crate::Config::Liveu as CLiveu;
use crate::config::Liveu as CLiveu;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

const APPLICATION_ID: &str = "SlZ3SHqiqtYJRkF0zO";

#[derive(Error, Debug)]
pub enum LiveuError {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Json error: {0}")]
    Json(#[from] serde_json::error::Error),
}

#[derive(Deserialize)]
struct Res {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    response: AuthRes,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthRes {
    access_token: String,
    expires_in: u32,
}

#[derive(Debug)]
pub struct Liveu {
    user_sesson: String,
    auth: AuthRes,
    user: CLiveu,
}

#[derive(Deserialize, Debug)]
pub struct Unit {
    id: String,
    reg_code: String,
}

#[derive(Deserialize, Debug)]
pub struct Inventories {
    pub units: Option<Vec<Unit>>,
}

impl Liveu {
    pub async fn authenticate(liveu_config: CLiveu) -> Result<Self, LiveuError> {
        let user_session = Uuid::new_v4();

        let client = reqwest::Client::new();
        let res = client
            .post("https://solo-api.liveu.tv/v0_prod/zendesk/userlogin")
            .basic_auth(&liveu_config.email, Some(&liveu_config.password))
            .header(ACCEPT, "application/json, text/plain, */*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
            .header(CONTENT_TYPE, "application/json;charset=UTF-8")
            .header(
                "x-user-name",
                format!("{}{}", &liveu_config.email, &user_session.to_string()),
            )
            .body(r#"{"return_to":"https://solo.liveu.tv/#/dashboard/units"}"#)
            .send()
            .await?
            .json::<Res>()
            .await?;

        Ok(Self {
            user_sesson: user_session.to_string(),
            auth: res.data.response,
            user: liveu_config,
        })
    }

    pub async fn get_inventories(&self) -> Result<Inventories, LiveuError> {
        let client = reqwest::Client::new();
        let res = client
            .get("https://lu-central.liveu.tv/luc/luc-core-web/rest/v0/inventories")
            .header(ACCEPT, "application/json, text/plain, */*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
            .header(AUTHORIZATION, format!("Bearer {}", self.auth.access_token))
            .header("application-id", APPLICATION_ID)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let res_json: Value = res.json().await?;

                Ok(serde_json::from_value::<Inventories>(
                    res_json["data"]["inventories"][0].to_owned(),
                )?)
            }
            StatusCode::NO_CONTENT => Ok(Inventories { units: None }),
            _ => panic!("ERROR GET_INVENTORIES report this to 715209"),
        }
    }

    //async fn refresh_token(&self) {}
}
