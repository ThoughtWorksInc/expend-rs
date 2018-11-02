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
    Payload(String, serde_json::Value),
}

#[derive(Serialize, Deserialize)]
pub struct Context {
    pub project: String,
    pub email: String,
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
        Payload(pt, p) => (pt, p),
    };
    pre_execute(&payload_type, &payload)?;
    client.post(&payload_type, payload)
}
