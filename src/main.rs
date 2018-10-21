#[macro_use]
extern crate failure;
extern crate expend;
extern crate failure_tools;

use std::{env, fs::File};
use failure_tools::ok_or_exit;
use failure::{Error, ResultExt};

fn run() -> Result<(), Error> {
    let filename = env::args().nth(1).ok_or_else(|| {
        format_err!(
            "USAGE: {} <input>\n\nWhere <input> is the input file with statements",
            env::args().next().expect("program name")
        )
    })?;
    let input_stream = File::open(&filename)
        .with_context(|_| format_err!("Could not open '{}' for reading", filename))?;

    expend::fun()
}

fn main() {
    ok_or_exit(run())
}
