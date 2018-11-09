extern crate expend;
extern crate pact_consumer;
extern crate pact_matching;
extern crate serde_json;

use expend::expensify;
use pact_consumer::builders::InteractionBuilder;
use pact_consumer::prelude::*;
use pact_matching::models::{Pact, PactSpecification};
use std::str::FromStr;

const ERR_RESPONSE: &str = include_str!("./fixtures/err-response.json");
const OK_RESPONSE: &str = include_str!("./fixtures/ok-response.json");
const EXPECTED_REQUEST_BODY: &str = include_str!("./fixtures/client-post.txt");

fn new_pact(make_interactions: impl FnOnce(&mut InteractionBuilder)) -> Pact {
    PactBuilder::new("expend", "expensify")
        .interaction("post something", make_interactions)
        .build()
}

fn write_pact_file(pact: &Pact, prefix: &str) {
    let pact_file = &std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join("fixtures")
        .join("pacts")
        .join(format!("{}-{}", prefix, pact.default_file_name()));
    std::fs::remove_file(pact_file).ok();
    pact.write_pact(pact_file, PactSpecification::V3).unwrap();
}

#[test]
fn expensify_post_failure() {
    let pact = new_pact(|i| {
        i.given("invalid credentials and valid input");
        i.request.post().path(expensify::ENDPOINT);
        i.response
            .status(200)
            .content_type("application/json")
            .body(ERR_RESPONSE);
    });
    let expensify_mock = pact.start_mock_server();

    let client = expensify::Client::new(
        Some(expensify_mock.url().clone()),
        "username",
        "invalid-password",
    );
    assert!(
        client
            .post(
                "some-type",
                serde_json::Value::from_str(r#"{"hello": 42}"#).unwrap()
            )
            .is_err()
    );

    write_pact_file(&pact, "failure");
}

#[test]
fn expensify_post_success() {
    let pact = new_pact(|i| {
        i.given("valid credentials and valid input");
        i.request
            .post()
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(EXPECTED_REQUEST_BODY)
            .path(expensify::ENDPOINT);
        i.response
            .status(200)
            .content_type("application/json")
            .body(OK_RESPONSE);
    });
    let expensify_mock = pact.start_mock_server();

    let client = expensify::Client::new(Some(expensify_mock.url().clone()), "username", "password");
    assert_eq!(
        client
            .post(
                "some-type",
                serde_json::Value::from_str(r#"{"hello": 42}"#).unwrap()
            )
            .unwrap(),
        serde_json::Value::from_str(OK_RESPONSE).unwrap()
    );

    write_pact_file(&pact, "success");
}
