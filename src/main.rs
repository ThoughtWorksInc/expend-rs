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

mod credentials;
mod options;

use failure::{bail, format_err, Error, ResultExt};
use failure_tools::ok_or_exit;
use options::*;
use std::path::Path;
use std::{
    fs::{create_dir_all, read_dir, File},
    io::{stdin, stdout},
    path::PathBuf,
};

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

fn into_context_dir(directory: Option<PathBuf>) -> Result<PathBuf, Error> {
    directory
        .or_else(|| {
            dirs::config_dir().map(|mut d| {
                d.push("expend-rs");
                d
            })
        }).ok_or_else(|| format_err!("Could not find configuration directory"))
}

fn context_file(directory: &Path, name: &str) -> PathBuf {
    directory.join(format!("{}.json", name))
}

fn handle_context(from: Option<PathBuf>, cmd: ContextSubcommand) -> Result<(), Error> {
    let config_dir = into_context_dir(from)?;
    Ok(match cmd {
        ContextSubcommand::Get { name } => {
            let ctx = context_from_file(&context_file(&config_dir, &name))?;
            serde_yaml::to_writer(stdout(), &ctx)?;
        }
        ContextSubcommand::Set {
            name,
            project,
            email,
        } => {
            let config_dir = config_dir;
            create_dir_all(&config_dir).with_context(|_| {
                format!(
                    "Could not create configuration directory at '{}'",
                    config_dir.display()
                )
            })?;

            let context_file = context_file(&config_dir, &name);

            let context = expend::Context { project, email };
            serde_json::to_writer_pretty(
                File::create(&context_file).with_context(|_| {
                    format!("Failed to open file at '{}'", context_file.display())
                })?,
                &context,
            )?;
            println!("Context '{}' set successfully", name);
        }

        ContextSubcommand::List => {
            if !config_dir.is_dir() {
                bail!("No contexts created - use 'context set' to create one.");
            }

            let mut count = 0;
            for stem in read_dir(&config_dir)?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter_map(|p: PathBuf| match p.extension() {
                    Some(ext) if ext == "json" => Some(p.clone()),
                    _ => None,
                }).filter_map(|p| path_to_context_name(&p))
            {
                println!("{}", stem);
                count += 1;
            }
            if count == 0 {
                bail!("Did not find a single contet. Create one using 'context set'.");
            }
        }
    })
}

fn path_to_context_name(file: &Path) -> Option<String> {
    file.file_stem().map(|s| s.to_string_lossy().into_owned())
}

fn context_from_file(file: &Path) -> Result<expend::Context, Error> {
    Ok(serde_json::from_reader(File::open(&file).with_context(
        |_| {
            format!(
                "Could not read context file at '{}'. Use 'context set \"{}\"' to create one.",
                file.display(),
                path_to_context_name(file).unwrap_or_else(|| "default".to_owned())
            )
        },
    )?)?)
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

            let context_dir = into_context_dir(post.context_from)?;

            let cmd = match post.cmd {
                PostSubcommands::PerDiem { context, kind: _ } => {
                    let _context = context_from_file(&context_file(&context_dir, &context))?;
                    unimplemented!()
                }
                PostSubcommands::FromFile {
                    context,
                    payload_type,
                    input,
                } => {
                    let context = context.map(|c| context_file(&context_dir, &c));
                    let context: Option<expend::Context> = match context {
                        Some(file) => Some(context_from_file(&file)?),
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
            handle_context(from, cmd)?;
            std::process::exit(0);
        }
    })
}

fn main() {
    ok_or_exit(run())
}
