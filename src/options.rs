extern crate chrono;

use self::chrono::{Date, Utc};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "structopt::clap::AppSettings::SubcommandRequired"))]
pub enum Args {
    #[structopt(name = "post")]
    /// Load a file with structured data and use it as payload
    Post(Post),
    #[structopt(name = "context", alias = "contexts")]
    /// Interact with contexts - one or more sets of properties that are shared across many sub-commands
    Context(Context),
}

#[derive(StructOpt)]
pub struct Post {
    #[structopt(long = "user-id", short = "u")]
    /// The user id, see https://integrations.expensify.com/Integration-Server/doc/#authentication
    pub user_id: Option<String>,
    #[structopt(long = "user-secret", short = "s")]
    /// The user secret, see https://integrations.expensify.com/Integration-Server/doc/#authentication
    pub user_secret: Option<String>,

    #[structopt(long = "auto-confirm", short = "y")]
    /// If set, we will not prompt prior to making the post to expensify.
    /// Mutually exclusive with '-n'
    pub yes: bool,
    #[structopt(long = "dry-run", short = "n")]
    /// If set, no action will be performed, but it will print what would be performed
    /// Mutually exclusive with '-y'
    pub dry_run: bool,

    #[structopt(long = "no-keychain")]
    /// If set, we will not use the keychain to retrieve previously entered credentials, nor will we write
    /// entered credentials to the keychain.
    pub no_keychain: bool,

    #[structopt(long = "clear-keychain-entry")]
    /// If set, the previously stored credentials will be cleared. This is useful if your credentials change.
    pub clear_keychain_entry: bool,

    #[structopt(parse(from_os_str), long = "context-dir")]
    /// The directory from which to load contexts.
    /// Defaults to your <OS config dir>/expend-rs
    pub context_from: Option<PathBuf>,

    #[structopt(
        parse(try_from_str = "expend::from_date_string"),
        long = "weekdate",
        alias = "w"
    )]
    /// The date of a day in the week that your per-diem dates should assume, formatted
    /// like 2018-09-25.
    pub weekdate: Option<Date<Utc>>,

    #[structopt(subcommand)]
    pub cmd: PostSubcommands,
}

#[derive(StructOpt)]
pub enum PostSubcommands {
    #[structopt(name = "per-diem")]
    /// Post a per-diem, relative to the current week, by default
    PerDiem {
        #[structopt(long = "context", short = "c", default_value = "default")]
        /// The name of the context to use.
        context: String,

        /// The kind of per-diem to file. Can be one of the following
        /// weekdays
        ///   - Monday to Friday
        kind: String,
    },
    #[structopt(name = "from-file")]
    /// Load a file with structured data and use it as payload.
    FromFile {
        #[structopt(long = "context", short = "c")]
        /// The name of the context to use. If unset, the context values have to be provided by the user.
        context: Option<String>,

        #[structopt(parse(from_os_str))]
        /// A path to the json or yaml file to load
        input: PathBuf,

        #[structopt(default_value = "create")]
        /// The kind of payload, corresponds to the expensify 'type of job' to execute.
        payload_type: String,
    },
}

#[derive(StructOpt)]
pub struct Context {
    #[structopt(parse(from_os_str), long = "from", alias = "at")]
    /// The directory in which we should look for serialized context information, or to which to write them.
    /// Defaults to your <OS config dir>/expend-rs
    pub from: Option<PathBuf>,

    #[structopt(subcommand)]
    pub cmd: ContextSubcommand,
}

#[derive(StructOpt)]
pub enum ContextSubcommand {
    #[structopt(name = "list")]
    /// List all available named contexts
    List,
    #[structopt(name = "set")]
    /// Set the optionally named context to the given values
    Set {
        #[structopt(long = "name", short = "n", default_value = "default")]
        /// The name of the context.
        name: String,

        #[structopt(long = "project", short = "p")]
        /// The project identifier. It's exactly what you see when selecting the project in Expensify
        project: String,

        #[structopt(long = "email", short = "e")]
        /// The email address used to login to expensify.
        email: String,
    },

    #[structopt(name = "get")]
    Get {
        #[structopt(default_value = "default")]
        /// The name of the context to retrieve.
        name: String,
    },
}
