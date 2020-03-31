use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use uuid::Uuid;

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
}

impl Liveu {
    pub async fn authenticate(email: &str, password: &str) -> Result<Self, reqwest::Error> {
        let user_session = Uuid::new_v4();

        let client = reqwest::Client::new();
        let res = client
            .post("https://solo-api.liveu.tv/v0_prod/zendesk/userlogin")
            .basic_auth(email, Some(password))
            .header(CONTENT_TYPE, "application/json;charset=UTF-8")
            .header(ACCEPT, "application/json, text/plain, */*")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
            .header(
                "x-user-name",
                format!("{}{}", email, &user_session.to_string()),
            )
            .body(r#"{"return_to":"https://solo.liveu.tv/#/dashboard/units"}"#)
            .send()
            .await?
            .json::<Res>()
            .await?;

        Ok(Self {
            user_sesson: user_session.to_string(),
            auth: res.data.response,
        })
    }

    // TODO: Implement
    // pub async fn getInventories(&self) -> Result<AuthRes, reqwest::Error> {}
}
