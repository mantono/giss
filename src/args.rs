use clap::{App, Arg, ArgMatches};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub fn parse_args(current_repo: &str) -> ArgMatches {
    let target_regex = Regex::new(r"^[\w\-]+(/[\w\-_\.]+)?$").unwrap();
    let target = Arg::with_name("target")
        .takes_value(true)
        .multiple(true)
        .help("Name of target(s)")
        .long_help("Name of the targets for the action. Can be a combination of one or several repositories, organizations or users. Any repository specified must be qualified with the owner or organization name. For example 'org/repo'. When no target is specified, repository in current directory will be used, if possible.")
        .validator(move |i| {
            if target_regex.is_match(&i) {
                Ok(())
            } else {
                Err(format!("Invalid target pattern: '{}'", i))
            }
        })
        .default_value(&current_repo);

    let token = Arg::with_name("token")
        .takes_value(true)
        .long("token")
        .short("t")
        .help("GitHub API token")
        .env("GITHUB_TOKEN");

    let assigned = Arg::with_name("assigned")
        .long("assigned")
        .short("a")
        .conflicts_with("repo")
        .help("Filter issues/pull requests assigned to user")
        .long_help("Only include issues and pull requests assigned to user");

    let limit = Arg::with_name("limit")
        .takes_value(true)
        .long("limit")
        .short("n")
        .help("Limit the number of issues listed")
        .long_help("Limit how many issues that should be listed")
        .default_value("10");

    let open = Arg::with_name("open")
        .long("open")
        .short("o")
        .conflicts_with("closed")
        .conflicts_with("all")
        .help("Only show open issues and pull requests")
        .long_help("Only show issues and pull requests in state open. This is enabled by default");

    let closed = Arg::with_name("closed")
        .long("closed")
        .short("c")
        .conflicts_with("open")
        .conflicts_with("all")
        .help("Only show closed issues and pull requests")
        .long_help("Only show issues and pull requests in state closed or merged");

    let all = Arg::with_name("all")
        .long("all")
        .short("A")
        .conflicts_with("open")
        .conflicts_with("closed")
        .help("Show all issues and pull requests, regardless of state")
        .long_help("Show all issues and pull requests and do not filter by open or closed state");

    let issues = Arg::with_name("issues")
        .long("issues")
        .short("i")
        .conflicts_with("review requests")
        .conflicts_with("pull requests")
        .help("Only list issues");

    let pull_requests = Arg::with_name("pull requests")
        .long("pull-requests")
        .short("p")
        .conflicts_with("review requests")
        .conflicts_with("issues")
        .help("Only list pull requests");

    let review_requests = Arg::with_name("review requests")
        .long("review-requests")
        .short("r")
        .conflicts_with("pull requests")
        .conflicts_with("issues")
        .conflicts_with("assigned")
        .help("Show pull requests where user is request to review")
        .long_help("Only show pull requests where the user has been requested to review it");

    let colors = Arg::with_name("colors")
        .long("colors")
        .short("C")
        .help("Enable colors")
        .long_help("Enable output with colors");

    let verbosity = Arg::with_name("verbosity")
        .takes_value(true)
        .default_value("1")
        .validator(|n: String| {
            let range = 0u8..=5u8;
            let n: u8 = n.parse::<u8>().unwrap();
            if range.contains(&n) {
                Ok(())
            } else {
                Err("Invalid value".to_string())
            }
        })
        .short("v")
        .long("verbosity")
        .help("Set verbosity level, 0 - 5")
        .long_help("Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that logging level configured via RUST_LOG overrides this setting.");

    let debug = Arg::with_name("debug")
        .takes_value(false)
        .short("D")
        .long("debug")
        .help("Print debug information")
        .long_help("Print debug information about current build for binary, useful for when an issue is encountered and reported");

    let args: ArgMatches = App::new(crate_name!())
        .about("Command line tool for listing GitHub issues and pull requests")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(token)
        .arg(target)
        .arg(assigned)
        .arg(limit)
        .arg(open)
        .arg(closed)
        .arg(all)
        .arg(issues)
        .arg(pull_requests)
        .arg(review_requests)
        .arg(colors)
        .arg(verbosity)
        .arg(debug)
        .get_matches();

    args
}

pub fn read_repo_from_file() -> String {
    let current_path: &Path = Path::new(".");
    let repo_root: PathBuf = traverse(&current_path);
    let config_file: PathBuf = repo_root.join(".git").join("config");
    log::debug!("Using Git config file: '{:?}'", config_file);
    let file_content: String = fs::read_to_string(config_file).expect("Could not find a git config");

    let lines: Vec<&str> = file_content.lines().filter(|f| f.contains("github.com")).collect();

    let repo: &str = lines
        .first()
        .expect("No Github repository found")
        .split_terminator(':')
        .last()
        .expect("No match");
    repo.trim_end_matches(".git").to_string()
}

fn traverse(path: &Path) -> PathBuf {
    let path_full: PathBuf = path
        .to_path_buf()
        .canonicalize()
        .expect("Could not create the canonical path");

    let git_config: PathBuf = path_full.join(".git").join("config");
    if git_config.exists() {
        return path_full;
    }
    match path_full.parent() {
        Some(parent) => traverse(parent),
        None => panic!("No .git directory found in hierarchy"),
    }
}
