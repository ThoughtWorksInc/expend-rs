#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use failure::Error;
use std::str::FromStr;

pub mod expensify;
pub mod types;

use types::{TransactionList, TransactionListElement};

pub enum Command {
    Payload(Option<Context>, String, serde_json::Value),
    PerDiem(Context, PerDiem),
}

#[derive(Serialize, Deserialize)]
pub struct Context {
    pub project: String,
    pub email: String,
}

impl From<(Context, PerDiem)> for TransactionList {
    fn from((ctx, kind): (Context, PerDiem)) -> Self {
        TransactionList {
            transaction_list_type: "expenses".to_owned(),
            employee_email: ctx.email.clone(),
            transaction_list: kind.into_transactions(&ctx),
        }
    }
}

fn apply_context(ctx: Context, mut payload: serde_json::Value) -> serde_json::Value {
    payload
        .get_mut("employeeEmail")
        .map(|v| *v = json!(ctx.email));
    payload
        .get_mut("transactionList")
        .and_then(serde_json::Value::as_array_mut)
        .map(|a| {
            for item in a.iter_mut() {
                item.get_mut("tag").map(|v| *v = json!(ctx.project.clone()));
            }
        });
    payload
}

pub fn execute(
    user_id: String,
    password: String,
    cmd: Command,
    pre_execute: impl FnOnce(&str, &serde_json::Value) -> Result<(), Error>,
) -> Result<serde_json::Value, Error> {
    use self::Command::*;

    let client = expensify::Client::new(None, user_id, password);
    let (payload_type, payload) = match cmd {
        Payload(None, pt, p) => (pt, p),
        Payload(Some(ctx), pt, mut p) => (pt, apply_context(ctx, p)),
        PerDiem(ctx, kind) => {
            let payload = serde_json::value::to_value(TransactionList::from((ctx, kind)))?;
            ("create".to_string(), payload)
        }
    };
    let payload = serde_json::value::to_value(payload)?;
    pre_execute(&payload_type, &payload)?;
    client.post(&payload_type, payload)
}

pub enum PerDiem {
    Weekdays,
}

impl FromStr for PerDiem {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use PerDiem::*;
        Ok(match s {
            "weekdays" => Weekdays,
            _ => bail!("Invalid per diem specification: '{}'", s),
        })
    }
}

impl PerDiem {
    fn into_transactions(self, ctx: &Context) -> Vec<TransactionListElement> {
        use PerDiem::*;
        let ts = Vec::new();
        match self {
            Weekdays => unimplemented!(),
        }
        ts
    }
}
