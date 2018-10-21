extern crate expend;
extern crate pact_consumer;
extern crate pact_matching;
extern crate serde_json;

use std::str::FromStr;
use pact_consumer::prelude::*;
use pact_consumer::builders::InteractionBuilder;
use pact_matching::models::{Pact, PactSpecification};
use expend::expensify;

const OK_RESPONSE: &str = include_str!("./fixtures/ok-response.json");
const EXPECTED_REQUEST_BODY: &str = include_str!("./fixtures/client-post.txt");

fn new_pact(make_interactions: impl FnOnce(&mut InteractionBuilder)) -> Pact {
    PactBuilder::new("expend", "expensify")
        .interaction("post any input settings", make_interactions)
        .build()
}

#[test]
fn expensify_pact() {
    let pact = new_pact(|i| {
        i.given("valid credentials and valid input");
        i.request
            .post()
            .body(EXPECTED_REQUEST_BODY)
            .path("/Integration-Server/ExpensifyIntegrations");
        i.response
            .status(200)
            .content_type("text/plain")
            .body(OK_RESPONSE);
    });
    let expensify_mock = pact.start_mock_server();

    let client = expensify::Client::new(Some(expensify_mock.url().clone()), "username", "password");
    assert_eq!(
        client
            .post(
                "some-type",
                serde_json::from_str(r#"{"hello": 42}"#).unwrap()
            )
            .unwrap(),
        serde_json::Value::from_str(OK_RESPONSE).unwrap()
    );

    write_pact_file(&pact);
}

fn write_pact_file(pact: &Pact) {
    let pact_file = &std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join("fixtures")
        .join("pacts")
        .join(pact.default_file_name());
    std::fs::remove_file(pact_file).ok();
    pact.write_pact(pact_file, PactSpecification::V3).unwrap();
}
