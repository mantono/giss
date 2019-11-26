#[macro_use]
extern crate clap;
extern crate dirs;
extern crate regex;

mod args;
mod create;
mod github_resources;
mod issue;
mod list;

use args::args::{parse_args, read_repo_from_file};
use create::create::{create_issue, read_issue};
use issue::issue::IssueRequest;
use itertools::Itertools;
use list::list::{list_issues, FilterConfig};

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    let current_repo: String = read_repo_from_file();
    let args: clap::ArgMatches = parse_args(&current_repo);

    let action: &str = args.value_of("action").expect("Action must be present");
    let token: String = args
        .value_of("token")
        .expect("No token was present")
        .to_string();

    let targets: Vec<&str> = args
        .values_of("target")
        .expect("Target must be present")
        .collect();

    match action {
        "create" => {
            if targets.len() > 1 {
                panic!("Multiple targets cannot be given when creating an issue")
            }
            let target: String = targets
                .first()
                .expect("At least one target must be present")
                .to_string();
            let read_issue: IssueRequest = read_issue(&target);
            create_issue(&target, &token, &read_issue)
        }
        "list" => {
            let targets: Vec<Target> =
                validate_targets(targets).expect("Must have valid target(s)");
            let config = FilterConfig::from_args(&args);
            list_issues(&targets, &token, &config)
        }
        _ => panic!("This should never happen"),
    }
}

pub enum Target {
    Organization { name: String },
    Repository { owner: String, name: String },
}

use Target::Organization as Org;
use Target::Repository as Repo;

impl Target {
    fn new(target: &str) -> Target {
        if target.contains("/") {
            let parts: Vec<&str> = target.split_terminator("/").collect();
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

    fn as_repo(&self) -> Option<(String, String)> {
        match self {
            Target::Organization { .. } => None,
            Target::Repository { owner, name } => Some((owner.clone(), name.clone())),
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

    let orgs: Vec<Target> = targets
        .iter()
        .filter(|t| !t.is_repo())
        .map(|t| t.clone())
        .collect();

    let repos: Vec<Target> = targets
        .iter()
        .filter(|t| t.is_repo())
        .map(|t| t.clone())
        .collect();

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
