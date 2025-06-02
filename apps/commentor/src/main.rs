use std::{
    fs::{create_dir_all, File},
    io::{stdin, BufWriter, Write},
    path::{Path, PathBuf},
    process::{self},
};

use clap::Parser;
use directories::ProjectDirs;
use github::{get_pull_request, parse_url, post_comment};
use parser::read_config_file;
use reqwest::blocking::Client;

mod github;
mod parser;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "*** Commentor ***\n\nA small tool so I don't have to type github comments manually anymore.\n\nSpecify comments below `comments:` in the config file, one per line. Lines beginning with # are ignored."
)]
enum Command {
    /// Create configuration file
    Init {
        #[arg(short, long, default_value = "subl")]
        editor_command: String,

        github_token: String,
    },

    /// Open configuration file in specified editor
    Open,

    /// Post comments from configuration file
    Run,
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        // .with_module_level("commentor", log::LevelFilter::Info) // TODO: Uncomment this
        // .with_module_level("commentor", log::LevelFilter::Trace) // TODO: Comment this, maybe
        .without_timestamps()
        .init()
        .unwrap();

    if let Some(directories) = ProjectDirs::from("", "", "commentor") {
        static CONFIG_FILE_NAME: &str = "commentor";

        let data_dir = directories.data_dir();
        let mut config_file_path =
            PathBuf::with_capacity(data_dir.as_os_str().len() + CONFIG_FILE_NAME.len());
        config_file_path.push(data_dir);
        config_file_path.push(CONFIG_FILE_NAME);
        log::trace!("Using config file path: {config_file_path:?}");

        /// Check if the config file exists, prompt the user to run init otherwise.
        fn initialised(config_file_path: &Path) -> bool {
            let exists = config_file_path.exists();

            if !exists {
                log::error!("Config file does not exist, please run `init` command.");
            }

            exists
        }

        match Command::parse() {
            Command::Init {
                editor_command,
                github_token,
            } => {
                if config_file_path.exists() {
                    log::warn!("Config file exists, overwriting...");
                } else {
                    log::trace!(
                        "Config file or directory not found, creating directory: {data_dir:?}"
                    );

                    if let Err(e) = create_dir_all(data_dir) {
                        log::error!("Failed to create data dir: {e}");
                    }
                }

                match File::create(config_file_path) {
                    Err(e) => log::error!("Failed to create config file: {e}"),
                    Ok(file) => {
                        let mut buffer = BufWriter::new(file);

                        if let Err(e) = writeln!(buffer, "editor: {editor_command}") {
                            log::error!("Failed to write to file: {e}");
                        }

                        if let Err(e) = writeln!(buffer, "github_token: {github_token}") {
                            log::error!("Failed to write to file: {e}");
                        }

                        if let Err(e) = writeln!(buffer, "pr_url:") {
                            log::error!("Failed to write to file: {e}");
                        }

                        if let Err(e) = writeln!(buffer, "comments:") {
                            log::error!("Failed to write to file: {e}");
                        }

                        if let Err(e) = buffer.flush() {
                            log::error!("Failed to flush write 6buffer: {e}");
                        }
                    },
                }

                log::info!("Config file initialised.");
            },

            Command::Open => {
                if initialised(&config_file_path) {
                    log::info!("Opening file: {config_file_path:?}");
                    match read_config_file(&config_file_path) {
                        Err(e) => log::error!("Failed to read config file: {e}"),
                        Ok(config) => {
                            process::Command::new(&config.editor_command)
                                .arg(config_file_path.as_path().as_os_str())
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                        },
                    }
                }
            },

            Command::Run => {
                if initialised(&config_file_path) {
                    match read_config_file(&config_file_path) {
                        Ok(config) => {
                            let client = Client::new();
                            match parse_url(&config.pr_url) {
                                Err(e) => log::error!("Failed to parse PR URL: {e}"),
                                Ok(identifier) => {
                                    log::info!("Looking up pr: {identifier:?}");

                                    match get_pull_request(
                                        &client,
                                        &identifier,
                                        &config.github_token,
                                    ) {
                                        Err(e) => {
                                            log::error!("Failed to get pull request details: {e}")
                                        },
                                        Ok(descriptor) => {
                                            if let Some(descriptor) = descriptor {
                                                println!();
                                                println!("**** Pull Request Found ****");
                                                println!("* URL: {:?}", descriptor.url);
                                                println!("* Title: {:?}", descriptor.title);
                                                println!("* Author: {:?}", descriptor.author);
                                                println!("* State: {:?}", descriptor.state);
                                                println!("****************************");

                                                println!();
                                                println!("The following comments will be posted:");

                                                for comment in config.comments.iter() {
                                                    println!("- {comment}");
                                                }

                                                println!();
                                                println!("Confirm? (y/n): ");

                                                let mut buf = String::with_capacity(10);
                                                stdin().read_line(&mut buf).unwrap();
                                                let trim = buf.trim();

                                                if trim == "y" || trim == "Y" {
                                                    for comment in config.comments.iter() {
                                                        print!("Posting comment {comment}: ");

                                                        match post_comment(
                                                            &client,
                                                            &identifier,
                                                            &config.github_token,
                                                            comment.trim(),
                                                        ) {
                                                            Ok(status) => println!("[{status:?}]"),
                                                            Err(e) => {
                                                                println!("[Failed with error {e}]")
                                                            },
                                                        }
                                                    }

                                                    println!();
                                                    println!("** Done **");
                                                } else {
                                                    log::error!("Aborted");
                                                }
                                            } else {
                                                log::error!(
                                                    "Pull request ({:?}) not found.",
                                                    config.pr_url
                                                );
                                            }
                                        },
                                    }
                                },
                            }
                        },
                        Err(e) => log::error!("Failed to read config file: {e}"),
                    }
                }
            },
        }
    } else {
        log::error!("Failed to get project directories.");
    }
}
