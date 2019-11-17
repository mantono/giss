#[macro_use]
extern crate clap;
extern crate regex;

mod args;
mod create;
mod issue;
mod list;

use args::args::{parse_args, read_repo_from_file};
use create::create::{create_issue, read_issue};
use issue::issue::IssueRequest;
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
            let config = FilterConfig::from_args(&args);
            list_issues(&targets, &token, &config)
        }
        _ => panic!("This should never happen"),
    }
}
