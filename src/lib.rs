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

pub mod context;
pub mod expensify;
pub mod perdiem;

use expensify::TransactionList;

const EXPENSIFY_DATE_FORMAT: &str = "%Y-%m-%d";

pub use context::{Context, UserContext};

pub enum Command {
    Payload(Option<Context>, String, serde_json::Value),
    PerDiem(Context, perdiem::TimePeriod, perdiem::Kind),
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
        Payload(Some(ctx), pt, mut p) => (pt, ctx.user.apply_to_value(p)),
        PerDiem(ctx, period, kind) => {
            let payload =
                serde_json::value::to_value(TransactionList::from_per_diem(ctx, period, kind)?)?;
            ("create".to_string(), payload)
        }
    };
    let payload = serde_json::value::to_value(payload)?;
    pre_execute(&payload_type, &payload)?;
    client.post(&payload_type, payload)
}

pub fn from_date_string(s: &str) -> Result<Date<Utc>, Error> {
    let date_string = format!("{}T00:00:00Z", s);
    Ok(date_string
        .parse::<DateTime<Utc>>()
        .with_context(|_| format!("Could not parse date string '{}'", date_string))?
        .date())
}
