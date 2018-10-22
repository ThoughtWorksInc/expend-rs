extern crate expend;
extern crate failure;
extern crate failure_tools;
#[macro_use]
extern crate structopt;

use structopt::StructOpt;
use failure_tools::ok_or_exit;
use failure::Error;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "make-cookie")]
struct Args {
    #[structopt(long = "user-id", short = "u")]
    /// The user id, see https://integrations.expensify.com/Integration-Server/doc/#authentication
    user_id: Option<String>,
    #[structopt(long = "user-secret", short = "s")]
    /// The user secret, see https://integrations.expensify.com/Integration-Server/doc/#authentication
    user_secret: Option<String>,

    #[structopt(short = "y")]
    /// If set, we will not prompt prior to making the post to expensify.
    /// Mutually exclusive with '-n'
    yes: bool,
    #[structopt(short = "n")]
    /// If set, no action will be performed, but it will print what would be performed
    /// Mutually exclusive with '-y'
    dry_run: bool,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Create,
}

#[derive(StructOpt)]
enum Create {
    #[structopt(name = "from-file")]
    FromFile(FromFile),
}

#[derive(StructOpt)]
struct FromFile {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn run() -> Result<(), Error> {
    let opt: Args = Args::from_args();
    match (&opt.user_id, &opt.user_secret) {
        (&Some(ref user), &Some(ref secret)) => unimplemented!(),
        (&Some(_), &None) => unimplemented!(),
        (&None, &Some(_)) => unimplemented!(),
        _ => unimplemented!(),
    }
    expend::fun()
}

fn main() {
    ok_or_exit(run())
}
