use failure::{bail, format_err, Error, ResultExt};
use options::ContextSubcommand;
use std::{fs::{create_dir_all, read_dir, File}, io::stdout, path::{Path, PathBuf}};

pub fn into_directory_path(directory: Option<PathBuf>) -> Result<PathBuf, Error> {
    directory
        .or_else(|| {
            dirs::config_dir().map(|mut d| {
                d.push("expend-rs");
                d
            })
        })
        .ok_or_else(|| format_err!("Could not find configuration directory"))
}

pub fn file_path(directory: &Path, name: &str) -> PathBuf {
    directory.join(format!("{}.json", name))
}

pub fn handle(from: Option<PathBuf>, cmd: ContextSubcommand) -> Result<(), Error> {
    use expend::{Categories, Category, Tag, Tags, UserContext};
    let config_dir = into_directory_path(from)?;
    match cmd {
        ContextSubcommand::Get { name } => {
            let config_file = file_path(&config_dir, &name);
            let ctx = from_file_path(&config_file)?;
            println!("Showing context at '{}'", config_file.display());
            serde_yaml::to_writer(stdout(), &ctx)?;
            println!();
        }
        ContextSubcommand::Set {
            name,
            project,
            email,
            country,
            travel_tag_name,
            travel_unbillable,
            category_per_diems_name,
        } => {
            let config_dir = config_dir;
            create_dir_all(&config_dir).with_context(|_| {
                format!(
                    "Could not create configuration directory at '{}'",
                    config_dir.display()
                )
            })?;

            let context_file = file_path(&config_dir, &name);

            let context = UserContext {
                project,
                email,
                country: country.parse()?,
                categories: Categories {
                    per_diems: Category {
                        name: category_per_diems_name,
                    },
                },
                tags: Tags {
                    travel: Tag {
                        name: travel_tag_name,
                        billable: !travel_unbillable,
                    },
                },
            };
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
                })
                .filter_map(|p| path_to_context_name(&p))
            {
                println!("{}", stem);
                count += 1;
            }
            if count == 0 {
                bail!("Did not find a single contet. Create one using 'context set'.");
            }
        }
    }
    Ok(())
}

fn path_to_context_name(file: &Path) -> Option<String> {
    file.file_stem().map(|s| s.to_string_lossy().into_owned())
}

pub fn from_file_path(file: &Path) -> Result<expend::UserContext, Error> {
    Ok(serde_json::from_reader(File::open(&file).with_context(|_| {
        format!(
            "Could not read context file at '{}'. Use 'context set \"{}\"' to create one.",
            file.display(),
            path_to_context_name(file).unwrap_or_else(|| "default".to_owned())
        )
    })?).with_context(|_| {
        format!("Could not deserialize context file at '{}'. You can try to recreate it with 'context set'.", file.display())
    })?)
}
