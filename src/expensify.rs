use failure::{self, ResultExt};
use reqwest::{self, Url};
use serde::Serialize;
use serde_json as json;
use std::str::FromStr;

pub const ENDPOINT: &str = "/Integration-Server/ExpensifyIntegrations";

pub struct Client {
    host: Url,
    username: String,
    password: String,
}

fn into_err(code: u16, value: json::Value) -> failure::Error {
    let value_str = json::to_string_pretty(&value).expect("valid json");
    format_err!("Request failed with http status {}: {}", code, value_str)
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

        let request_payload = json!({
            "type": request_type.to_owned(),
            "credentials": {
                "partnerUserID": self.username.clone(),
                "partnerUserSecret": self.password.clone(),
            },
            "inputSettings": json::to_value(input)?,
        });

        let json_str = json::to_string(&request_payload)?;
        let params = [("requestJobDescription", &json_str)];

        let mut response = reqwest::Client::new()
            .post(url)
            .form(&params)
            .send()
            .context("Post request failed")?;
        let value: json::Value = response.json().context("failed to parse body as json")?;

        if response.status().is_success() {
            match value.get("responseCode").and_then(|v| v.as_u64()) {
                Some(code) if code < 200 || code >= 300 => Err(into_err(code as u16, value)),
                _ => Ok(value),
            }
        } else {
            Err(into_err(response.status().as_u16(), value))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransactionList {
    #[serde(rename = "type")]
    pub transaction_list_type: String,

    #[serde(rename = "employeeEmail")]
    pub employee_email: String,

    #[serde(rename = "transactionList")]
    pub transaction_list: Vec<TransactionListElement>,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionListElement {
    #[serde(rename = "created")]
    pub created: String,

    #[serde(rename = "currency")]
    pub currency: String,

    #[serde(rename = "merchant")]
    pub merchant: String,

    #[serde(rename = "amount")]
    pub amount: i32,

    #[serde(rename = "category")]
    pub category: String,

    #[serde(rename = "tag")]
    pub tag: String,

    #[serde(rename = "billable")]
    pub billable: bool,

    #[serde(rename = "reimbursable")]
    pub reimbursable: bool,

    #[serde(rename = "comment")]
    pub comment: String,
}
