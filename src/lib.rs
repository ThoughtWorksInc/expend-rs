#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use failure::Error;

pub mod expensify;

pub fn fun() -> Result<(), Error> {
    unimplemented!();
}
