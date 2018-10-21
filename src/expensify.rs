use reqwest::{self, Url};
use serde_json as json;
use std::str::FromStr;
use failure::{self, ResultExt};

const ENDPOINT: &str = "/Integration-Server/ExpensifyIntegrations";

pub struct Client {
    host: Url,
    username: String,
    password: String,
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
    pub fn post(&self, input: &json::Value) -> Result<json::Value, failure::Error> {
        let url = self.host
            .join(ENDPOINT)
            .expect("parsing of static endpoint");
        Ok(reqwest::Client::new()
            .post(url)
            .json(input)
            .send()
            .context("Post request failed")?
            .json()
            .context("failed to parse body as json")?)
    }
}
