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

use failure::{bail, format_err, Error, ResultExt};
use failure_tools::ok_or_exit;
use keyring::Keyring;
use std::{
    convert::From,
    fs::{create_dir_all, read_dir, File},
    io::{stderr, stdin},
    path::PathBuf,
    str::FromStr,
};
use structopt::StructOpt;
use termion::input::TermRead;

#[derive(Serialize, Deserialize)]
struct Credentials {
    user_id: String,
    user_secret: String,
}

impl From<(String, String)> for Credentials {
    fn from(f: (String, String)) -> Self {
        Credentials {
            user_id: f.0,
            user_secret: f.1,
        }
    }
}

impl From<Credentials> for (String, String) {
    fn from(c: Credentials) -> Self {
        (c.user_id, c.user_secret)
    }
}

impl FromStr for Credentials {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

#[derive(StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::SubcommandRequired"))]
enum Args {
    #[structopt(name = "post")]
    /// Load a file with structured data and use it as payload
    Post(Post),
    #[structopt(name = "context", alias = "contexts")]
    /// Interact with contexts - one or more sets of properties that are shared across many sub-commands
    Context(Context),
}

#[derive(StructOpt)]
struct Post {
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

    #[structopt(long = "no-keychain")]
    /// If set, we will not use the keychain to retrieve previously entered credentials, nor will we write
    /// entered credentials to the keychain.
    no_keychain: bool,

    #[structopt(long = "clear-keychain-entry")]
    /// If set, the previously stored credentials will be cleared. This is useful if your credentials change.
    clear_keychain_entry: bool,

    #[structopt(subcommand)]
    cmd: PostSubcommands,
}

#[derive(StructOpt)]
enum PostSubcommands {
    #[structopt(name = "from-file")]
    /// Load a file with structured data and use it as payload
    FromFile {
        #[structopt(parse(from_os_str))]
        /// A path to the json or yaml file to load
        input: PathBuf,

        #[structopt(default_value = "create")]
        /// The kind of payload, corresponds to the expensify 'type of job' to execute.
        payload_type: String,
    },
}

#[derive(StructOpt)]
struct Context {
    #[structopt(parse(from_os_str), long = "from", alias = "at")]
    /// The directory in which we should load for serialized context information.
    /// Defaults to your <OS config dir>/expend-rs
    from: Option<PathBuf>,

    #[structopt(subcommand)]
    cmd: ContextSubcommand,
}

#[derive(StructOpt)]
enum ContextSubcommand {
    #[structopt(name = "list")]
    /// List all available named contexts
    List,
    #[structopt(name = "set")]
    /// Set the optionally named context to the given values
    Set(SetContext),
}

#[derive(StructOpt)]
struct SetContext {
    #[structopt(long = "name", short = "n", default_value = "default")]
    /// The name of the context.
    name: String,

    #[structopt(long = "project", short = "p")]
    /// The project identifier. It's exactly what you see when selecting the project in Expensify
    project: String,

    #[structopt(long = "email", short = "e")]
    /// The email address used to login to expensify.
    email: String,
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

fn get_or_clear_credentials_from_keychain(clear: bool) -> Result<Option<(String, String)>, Error> {
    let username = username::get_user_name()?;
    let keyring = Keyring::new("expend-rs cli", &username);
    if clear {
        eprintln!("Clearing previously stored credentials");
        keyring.delete_password().ok();
        Ok(None)
    } else {
        eprintln!("Trying to use previously saved credentials from keychain.");
        let credentials: Credentials = match keyring.get_password() {
            Ok(pw) => pw.parse()?,
            Err(_) => return Ok(None),
        };
        Ok(Some(credentials.into()))
    }
}

fn store_credentials_in_keychain(creds: (String, String)) -> Result<(String, String), Error> {
    let username = username::get_user_name()?;
    let keyring = Keyring::new("expend-rs cli", &username);
    let creds: Credentials = creds.into();
    let creds_str = serde_json::to_string(&creds)?;
    keyring.set_password(&creds_str)?;
    Ok(creds.into())
}

fn query_credentials_from_user() -> Result<(String, String), Error> {
    eprint!("Please enter your user user-id: ");
    let mut user_id = String::new();
    stdin().read_line(&mut user_id)?;

    eprint!("Please enter your user user secret (it won't display): ");
    let user_secret = stdin()
        .read_passwd(&mut stderr())?
        .ok_or_else(|| format_err!("Cannot proceed without a password."))?;
    eprintln!();
    Ok((user_id, user_secret))
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
    serde_yaml::to_writer(std::io::stdout(), &value)?;
    Ok(())
}

fn handle_context(from: Option<PathBuf>, cmd: ContextSubcommand) -> Result<i32, Error> {
    let config_dir = from
        .or_else(|| {
            dirs::config_dir().map(|mut d| {
                d.push("expend-rs");
                d
            })
        }).ok_or_else(|| format_err!("Could not find configuration directory"))?;

    match cmd {
        ContextSubcommand::Set(SetContext {
            name,
            project,
            email,
        }) => {
            let mut config_dir = config_dir;
            create_dir_all(&config_dir).with_context(|_| {
                format!(
                    "Could not create configuration directory at '{}'",
                    config_dir.display()
                )
            })?;

            config_dir.push(format!("{}.json", name));
            let context_file = config_dir;

            let context = expend::Context { project, email };
            serde_json::to_writer_pretty(
                File::create(&context_file).with_context(|_| {
                    format!("Failed to open file at '{}'", context_file.display())
                })?,
                &context,
            )?;
            println!("Context '{}' set successfully", name);
            Ok(0)
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
                }).filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            {
                println!("{}", stem);
                count += 1;
            }
            if count == 0 {
                bail!("Did not find a single contet. Create one using 'context set'.");
            }
            Ok(0)
        }
    }
}

fn run() -> Result<(), Error> {
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
                    get_or_clear_credentials_from_keychain(post.clear_keychain_entry)?
                } {
                    Some(creds) => creds,
                    None => query_credentials_from_user().and_then(|creds| {
                        if post.no_keychain {
                            Ok(creds)
                        } else {
                            eprintln!(
                                "Storing credentials in keychain - use --no-keychain to disable."
                            );
                            store_credentials_in_keychain(creds)
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

            let cmd = match post.cmd {
                PostSubcommands::FromFile {
                    payload_type,
                    input,
                } => {
                    let json_value: serde_json::Value =
                        serde_yaml::from_reader(std::fs::File::open(&input).with_context(
                            |_| format!("Failed to open file at '{}'", input.display()),
                        )?)?;
                    expend::Command::Payload(payload_type, json_value)
                }
            };

            expend::execute(user, secret, cmd, |type_name, value| {
                confirm_payload(mode, type_name, value)
            }).and_then(show_value)?
        }
        Args::Context(Context { from, cmd }) => {
            std::process::exit(handle_context(from, cmd)?);
        }
    })
}

fn main() {
    ok_or_exit(run())
}
