#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate chrono;
extern crate time;

use chrono::prelude::*;
use failure::{Error, ResultExt};
use std::str::FromStr;

pub mod expensify;
pub mod types;

use types::{TransactionList, TransactionListElement};

pub enum Command {
    Payload(Option<Context>, String, serde_json::Value),
    PerDiem(Context, Option<Date<Utc>>, PerDiem),
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
        PerDiem(ctx, _weekdate, kind) => {
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

const EXPENSIFY_DATE_FORMAT: &str = "%Y-%m-%d";

fn to_date_string(d: &Date<Utc>) -> String {
    d.format(EXPENSIFY_DATE_FORMAT).to_string()
}

pub fn from_date_string(s: &str) -> Result<Date<Utc>, Error> {
    let date_string = format!("{}T00:00:00Z", s);
    Ok(date_string
        .parse::<DateTime<Utc>>()
        .with_context(|_| format!("Could not parse date string '{}'", date_string))?
        .date())
}

impl PerDiem {
    fn into_transactions(self, ctx: &Context) -> Vec<TransactionListElement> {
        use time::Duration;
        use PerDiem::*;

        let mut ts = Vec::new();
        match self {
            Weekdays => {
                let this_weeks_monday = {
                    let d = Utc::today();
                    d.checked_sub_signed(Duration::days(d.weekday().num_days_from_monday() as i64))
                        .unwrap()
                };
                let this_weeks_friday = this_weeks_monday
                    .checked_add_signed(Duration::days(5 - 1))
                    .unwrap();

                ts.push(TransactionListElement {
                    created: to_date_string(&this_weeks_monday),
                    currency: String::new(),
                    merchant: String::new(),
                    amount: 0,
                    category: String::new(),
                    tag: ctx.project.clone(),
                    billable: false,
                    reimbursable: false,
                    comment: format!(
                        "{} to {}",
                        to_date_string(&this_weeks_monday),
                        to_date_string(&this_weeks_friday)
                    ),
                });
            }
        }
        ts
    }
}
