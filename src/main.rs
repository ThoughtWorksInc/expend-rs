extern crate expend;
extern crate failure;
extern crate failure_tools;
extern crate serde_json;
extern crate serde_yaml;
extern crate structopt;
extern crate termion;

use structopt::StructOpt;
use failure_tools::ok_or_exit;
use failure::{bail, Error, ResultExt};
use std::path::PathBuf;
use std::io::stdin;

#[derive(StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::SubcommandRequired"))]
struct Args {
    #[structopt(long = "user-id", short = "u")]
    /// The user id, see https://integrations.expensify.com/Integration-Server/doc/#authentication
    user_id: Option<String>,
    #[structopt(long = "user-secret", short = "s")]
    /// The user secret, see https://integrations.expensify.com/Integration-Server/doc/#authentication
    user_secret: Option<String>,

    #[structopt(long = "auto-confirm", short = "y")]
    /// If set, we will not prompt prior to making the post to expensify.
    /// Mutually exclusive with '-n'
    yes: bool,
    #[structopt(long = "dry-run", short = "n")]
    /// If set, no action will be performed, but it will print what would be performed
    /// Mutually exclusive with '-y'
    dry_run: bool,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "from-file")]
    /// Load a file with structured data and use it as payload
    FromFile(FromFile),
}

#[derive(StructOpt)]
struct FromFile {
    /// The kind of payload, corresponds to the expensify 'type of job' to execute
    payload_type: String,
    #[structopt(parse(from_os_str))]
    /// A path to the json or yaml file to load
    input: PathBuf,
}

pub enum Mode {
    DryRun,
    AutoConfirm,
    Confirm,
}

fn exit_with(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1)
}

fn credentials_from_keychain() -> Result<(String, String), Error> {
    unimplemented!("credentials from keychain");
}

fn query_credentials_from_user(_keychain_error: Error) -> Result<(String, String), Error> {
    unimplemented!("query user credentials");
}

fn confirm_payload(mode: Mode, type_name: &str, value: &serde_json::Value) -> Result<(), Error> {
    use Mode::*;
    println!(
        "The following '{}' payload would be sent to Expensify:",
        type_name
    );
    serde_yaml::to_writer(std::io::stdout(), value)?;
    println!("\n");

    Ok(match mode {
        DryRun => {
            bail!("Aborted before post due to dry-run mode.");
        }
        Confirm => {
            if !termion::is_tty(&stdin()) {
                bail!("Cannot prompt if stdin is not a tty. Use -y to auto-confirm the operation.");
            }

            eprintln!("Please type 'y' to post or anything else to cancel.");
            let mut buf = String::new();
            stdin().read_line(&mut buf)?;
            if buf.trim().to_ascii_lowercase() != "y" {
                bail!("Aborted by user");
            }
        }
        AutoConfirm => (),
    })
}

fn show_value(value: serde_json::Value) -> Result<(), Error> {
    println!("Expensify said:",);
    serde_yaml::to_writer(std::io::stdout(), &value)?;
    Ok(())
}

fn run() -> Result<(), Error> {
    let opt: Args = Args::from_args();
    let (user, secret) = match (&opt.user_id, &opt.user_secret) {
        (Some(ref user), Some(ref secret)) => (user.to_owned(), secret.to_owned()),
        (Some(_), None) => exit_with("Please provide the secret as well with --user-secret."),
        (None, Some(_)) => exit_with("Please provide the user as well with --user-id."),
        (None, None) => credentials_from_keychain().or_else(query_credentials_from_user)?,
    };

    let mode = match (opt.dry_run, opt.yes) {
        (true, true) => exit_with("--auto-confirm and --dry-run are mutually exclusive."),
        (true, false) => Mode::DryRun,
        (false, true) => Mode::AutoConfirm,
        (false, false) => Mode::Confirm,
    };

    let cmd = match opt.cmd {
        Command::FromFile(FromFile {
            payload_type,
            input,
        }) => {
            let json_value: serde_json::Value =
                serde_yaml::from_reader(std::fs::File::open(&input)
                    .with_context(|_| format!("Failed to open file at '{}'", input.display()))?)?;
            expend::Command::Payload(payload_type, json_value)
        }
    };

    expend::execute(user, secret, cmd, |type_name, value| {
        confirm_payload(mode, type_name, value)
    }).and_then(show_value)
}

fn main() {
    ok_or_exit(run())
}
