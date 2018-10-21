use reqwest::{self, Url};
use serde_json as json;
use serde::Serialize;
use std::str::FromStr;
use failure::{self, ResultExt};

pub const ENDPOINT: &str = "/Integration-Server/ExpensifyIntegrations";

pub struct Client {
    host: Url,
    username: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Credentials {
    partner_user_id: String,
    partner_user_secret: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpensifyRequest {
    #[serde(rename = "type")]
    type_: String,
    credentials: Credentials,
    input_settings: json::Value,
}

impl Client {
    pub fn new(
        host: Option<Url>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Client {
        Client {
            host: host.unwrap_or_else(|| {
                Url::from_str("https://integrations.expensify.com")
                    .expect("default url to be correct")
            }).into(),
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn post(
        &self,
        request_type: &str,
        input: impl Serialize,
    ) -> Result<json::Value, failure::Error> {
        let url = self.host
            .join(ENDPOINT)
            .expect("parsing of static endpoint");
        let request_payload = ExpensifyRequest {
            type_: request_type.to_owned(),
            credentials: Credentials {
                partner_user_id: self.username.clone(),
                partner_user_secret: self.password.clone(),
            },
            input_settings: json::to_value(input)?,
        };
        let json_str = json::to_string_pretty(&request_payload)?;
        let body_text = format!(r#"requestJobDescription={}"#, json_str);
        let mut response = reqwest::Client::new()
            .post(url)
            .header("Content-Type", "text/plain")
            .body(body_text)
            .send()
            .context("Post request failed")?;
        let value = response.json().context("failed to parse body as json")?;
        if response.status().is_success() {
            Ok(value)
        } else {
            Err(format_err!(
                "Request failed with http status {}: {}",
                response.status().as_u16(),
                value
            ))
        }
    }
}
