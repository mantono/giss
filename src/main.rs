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
use list::{list_issues, FilterConfig};
use logger::setup_logging;

const GITHUB_API_V3_URL: &str = "https://api.github.com";

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
    list_issues(&user, &targets, &token, &config)
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

    fn is_repo(&self) -> bool {
        match self {
            Target::Organization { .. } => false,
            Target::Repository { .. } => true,
        }
    }

    fn as_org(&self) -> Option<String> {
        match self {
            Target::Organization { name } => Some(name.clone()),
            Target::Repository { .. } => None,
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
    let orgs: Vec<Target> = targets.iter().filter(|t| !t.is_repo()).cloned().collect();
    let repos: Vec<Target> = targets.iter().filter(|t| t.is_repo()).cloned().collect();

    if targets.is_empty() {
        Result::Err("No targets specified")
    } else if !orgs.is_empty() && !repos.is_empty() {
        Result::Err("Cannot give organizations and repositories at the same time")
    } else if repos.len() > 1 {
        Result::Err("Cannot give multiple repositories")
    } else if !orgs.is_empty() {
        Ok(orgs)
    } else {
        Ok(repos)
    }
}
