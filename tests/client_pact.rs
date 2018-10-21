extern crate expend;
extern crate pact_consumer;
extern crate serde_json;

use std::str::FromStr;
use pact_consumer::prelude::*;
use expend::expensify;

const OK_RESPONSE: &str = include_str!("./fixtures/ok-response.json");

#[test]
fn expensify_pact() {
    // Define the Pact for the test (you can setup multiple interactions by chaining the given or upon_receiving calls)
    let pact = PactBuilder::new("expend", "expensify")
        .interaction("post any input settings", |i| {
            i.given("valid credentials and valid input");
            i.request
                .post()
                .path("/Integration-Server/ExpensifyIntegrations");
            i.response
                .status(200)
                .content_type("application/json")
                .body(OK_RESPONSE);
        })
        .build();
    let expensify_mock = pact.start_mock_server();
    let client = expensify::Client::new(Some(expensify_mock.url().clone()), "username", "password");
    assert_eq!(
        client
            .post(&serde_json::from_str(r#"{"hello": 42}"#).unwrap())
            .unwrap(),
        serde_json::Value::from_str(OK_RESPONSE).unwrap()
    );
}
