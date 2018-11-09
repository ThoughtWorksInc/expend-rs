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

    #[structopt(parse(try_from_str = "expend::from_date_string"), long = "weekdate", alias = "w")]
    /// The date of a day in the week that your per-diem dates should assume, formatted
    /// like 2018-09-25.
    pub weekdate: Option<Date<Utc>>,

    #[structopt(subcommand)]
    pub cmd: PostSubcommands,
}

#[derive(StructOpt)]
pub enum PostSubcommands {
    #[structopt(name = "per-diem", alias = "perdiem")]
    /// Post a per-diem, relative to the current week, by default
    PerDiem {
        #[structopt(long = "context", short = "c", default_value = "default")]
        /// The name of the context to use.
        context: String,

        /// The kind of per-diem to file.
        /// Valid values are:
        /// |weekdays
        ///     - Equivalent to Monday-Friday|
        /// |Mon OR Monday OR Tue OR Tuesday ... Sun OR Sunday
        ///     - the given singular day, either as shorthand or full identifier. Case-insensitive|
        /// |<day>,<day>[,<day>...]
        ///     - <day> can be any day like Mon or Monday. Days will be unified, and ordered, thus
        ///       duplicate and out-of-order days can not be expressed as they will be fixed automatically.
        time_period: String,

        #[structopt(raw(possible_values = r#"&["breakfast", "fullday", "arrival", "departure", "daytrip", "lunch", "dinner"]"#))]
        /// The kind of per diem you need.
        kind: String,

        #[structopt(long = "subtract", short = "s")]
        /// If set, all values will be negated, effectively subtracting them. Useful if you want to use a range for positive per-diems,
        /// and subtract individual lunches or dinners.
        subtract: bool,

        #[structopt(long = "comment", short = "m")]
        /// The comment to be used. It should explain the purpose of the per-diem.
        /// If the time period is not a single day, the comment will be added as suffix to comment generated using the dates '<from> to <to>'.
        comment: Option<String>,
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

        #[structopt(long = "country", short = "c", default_value = "Germany")]
        #[structopt(raw(possible_values = r#"&["Germany"]"#))]
        /// The name of the country you are in. It's used to identify your currency and currency symbol.
        country: String,

        #[structopt(long = "project", short = "p")]
        /// The project identifier. It's exactly what you see when selecting the project in Expensify
        project: String,

        #[structopt(long = "email", short = "e")]
        /// The email address used to login to expensify.
        email: String,

        #[structopt(long = "travel-tag-name", default_value = "Travel")]
        /// The name of the tag to use for travel related expenses.
        travel_tag_name: String,

        #[structopt(long = "travel-tag-unbillable")]
        /// If set, all travel expenses will be unbillable.
        travel_unbillable: bool,
    },

    #[structopt(name = "get")]
    Get {
        #[structopt(default_value = "default")]
        /// The name of the context to retrieve.
        name: String,
    },
}
