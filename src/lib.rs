#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use failure::Error;

pub mod expensify;

pub enum Command {
    Payload(Option<Context>, String, serde_json::Value),
}

#[derive(Serialize, Deserialize)]
pub struct Context {
    pub project: String,
    pub email: String,
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
    };
    pre_execute(&payload_type, &payload)?;
    client.post(&payload_type, payload)
}
