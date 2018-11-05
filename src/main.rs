extern crate expend;
extern crate failure;
extern crate failure_tools;
extern crate keyring;
#[macro_use]
extern crate serde_derive;
extern crate dirs;
extern crate serde_json;
extern crate serde_yaml;
extern crate structopt;
extern crate termion;
extern crate username;

mod context;
mod credentials;
mod options;

use failure::{bail, Error, ResultExt};
use failure_tools::ok_or_exit;
use options::*;
use std::io::{stdin, stdout};

pub enum Mode {
    DryRun,
    AutoConfirm,
    Confirm,
}

fn exit_with(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1)
}

fn confirm_payload(mode: Mode, type_name: &str, value: &serde_json::Value) -> Result<(), Error> {
    use Mode::*;
    println!(
        "The following '{}' payload would be sent to Expensify:",
        type_name
    );
    serde_yaml::to_writer(stdout(), value)?;
    println!("\n");

    Ok(match mode {
        DryRun => {
            bail!("Aborted before post due to dry-run mode.");
        }
        Confirm => {
            if !termion::is_tty(&stdin()) {
                bail!("Cannot prompt if stdin is not a tty. Use -y to auto-confirm the operation.");
            }

            eprint!("Please type 'y' to post or anything else to cancel: ");
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
    serde_yaml::to_writer(stdout(), &value)?;
    Ok(())
}

fn run() -> Result<(), Error> {
    use structopt::StructOpt;
    let opt: Args = Args::from_args();

    Ok(match opt {
        Args::Post(post) => {
            let (user, secret) = match (&post.user_id, &post.user_secret) {
                (Some(ref user), Some(ref secret)) => (user.to_owned(), secret.to_owned()),
                (Some(_), None) => {
                    exit_with("Please provide the secret as well with --user-secret.")
                }
                (None, Some(_)) => exit_with("Please provide the user as well with --user-id."),
                (None, None) => match if post.no_keychain {
                    None
                } else {
                    credentials::from_keychain_or_clear(post.clear_keychain_entry)?
                } {
                    Some(creds) => creds,
                    None => credentials::query_from_user().and_then(|creds| {
                        if post.no_keychain {
                            Ok(creds)
                        } else {
                            eprintln!(
                                "Storing credentials in keychain - use --no-keychain to disable."
                            );
                            credentials::store_in_keychain(creds)
                        }
                    })?,
                },
            };

            let mode = match (post.dry_run, post.yes) {
                (true, true) => exit_with("--auto-confirm and --dry-run are mutually exclusive."),
                (true, false) => Mode::DryRun,
                (false, true) => Mode::AutoConfirm,
                (false, false) => Mode::Confirm,
            };

            let context_dir = context::into_directory_path(post.context_from)?;

            let cmd = match post.cmd {
                PostSubcommands::PerDiem {
                    weekdate,
                    context,
                    kind,
                } => {
                    let context =
                        context::from_file_path(&context::file_path(&context_dir, &context))?;
                    let kind: expend::PerDiem = kind.parse()?;
                    expend::Command::PerDiem(context, weekdate, kind)
                }
                PostSubcommands::FromFile {
                    context,
                    payload_type,
                    input,
                } => {
                    let context = context.map(|c| context::file_path(&context_dir, &c));
                    let context: Option<expend::Context> = match context {
                        Some(file) => Some(context::from_file_path(&file)?),
                        None => None,
                    };

                    let json_value: serde_json::Value =
                        serde_yaml::from_reader(std::fs::File::open(&input).with_context(
                            |_| format!("Failed to open file at '{}'", input.display()),
                        )?)?;
                    expend::Command::Payload(context, payload_type, json_value)
                }
            };

            expend::execute(user, secret, cmd, |type_name, value| {
                confirm_payload(mode, type_name, value)
            }).and_then(show_value)?
        }
        Args::Context(Context { from, cmd }) => {
            context::handle(from, cmd)?;
            std::process::exit(0);
        }
    })
}

fn main() {
    ok_or_exit(run())
}
