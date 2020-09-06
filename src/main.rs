#[macro_use]
extern crate clap;
extern crate dirs;
extern crate lazy_static;
extern crate log;
extern crate regex;

mod api;
mod args;
mod github_resources;
mod issue;
mod list;
mod logger;
mod search;
mod user;

use args::{parse_args, read_repo_from_file};
use itertools::Itertools;
use list::FilterConfig;
use logger::setup_logging;
use std::fmt;

fn main() {
    let current_repo: String = read_repo_from_file();
    let args: clap::ArgMatches = parse_args(&current_repo);
    let verbosity_level: u8 = args.value_of("verbosity").unwrap().parse::<u8>().unwrap();
    setup_logging(verbosity_level);

    let token: String = args.value_of("token").expect("No token was present").to_string();
    let targets: Vec<&str> = args.values_of("target").expect("Target must be present").collect();
    let targets: Vec<Target> = validate_targets(targets).expect("Must have valid target(s)");
    let user: String = fetch_username(&token);
    let config = FilterConfig::from_args(&args);

    log::debug!("Config: {:?}", config);
    list::list_issues(&user, &targets, &token, &config)
}

pub enum Target {
    Organization { name: String },
    Repository { owner: String, name: String },
}

use crate::user::fetch_username;

impl Target {
    fn new(target: &str) -> Target {
        if target.contains('/') {
            let parts: Vec<&str> = target.split_terminator('/').collect();
            if parts.len() != 2 {
                panic!("Expected format 'org/repo', got: {}", target);
            }
            Target::Repository {
                owner: parts[0].to_string(),
                name: parts[1].to_string(),
            }
        } else {
            Target::Organization {
                name: target.to_string(),
            }
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Organization { name } => write!(f, "org:{}", name),
            Target::Repository { owner, name } => write!(f, "repo:{}/{}", owner, name),
        }
    }
}

impl Clone for Target {
    fn clone(&self) -> Target {
        match self {
            Target::Organization { name } => Target::Organization { name: name.clone() },
            Target::Repository { owner, name } => Target::Repository {
                owner: owner.clone(),
                name: name.clone(),
            },
        }
    }
}

fn validate_targets(targets: Vec<&str>) -> Result<Vec<Target>, &str> {
    let targets: Vec<Target> = targets.iter().unique().map(|t| Target::new(t)).collect();

    if targets.is_empty() {
        Result::Err("No targets specified")
    } else {
        Ok(targets)
    }
}
