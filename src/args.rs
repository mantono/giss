pub mod args {
    use clap::{App, Arg, ArgMatches};
    use regex::Regex;
    use std::fs;
    use std::ops::Deref;
    use std::path::{Path, PathBuf};

    pub fn parse_args(current_repo: &str) -> ArgMatches {
        let action = Arg::with_name("action")
            .default_value("list")
            .help("Action to take")
            .long_help("Choose whether to list issues or create a new one.")
            .possible_values(&["list", "create"]);

        let target_regex = Regex::new(r"^[\w\-]+(/[\w\-_\.]+)?$").unwrap();
        let target = Arg::with_name("target")
            .takes_value(true)
            .multiple(true)
            .help("Name of target(s)")
            .long_help("Name of the targets for the action. Can be either a single repository or one or multiple organizations or owners. Any repository specified must be qualified with the owner or organization name. For example 'org/repo'. If action is 'create' then only a repository will be accepted. When no target is specified, repository in current directory will be used, if possible.")
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
            .long_help(
                "Only show issues and pull requests in state open. This is enabled by default",
            );

        let closed = Arg::with_name("closed")
            .long("closed")
            .short("c")
            .conflicts_with("open")
            .conflicts_with("all")
            .help("Only show closed issues and pull requests")
            .long_help("Only show issues and pull requests in state closed");

        let all = Arg::with_name("all")
            .long("all")
            .short("A")
            .conflicts_with("open")
            .conflicts_with("closed")
            .help("Show all issues and pull requests, regardless of state")
            .long_help(
                "Show all issues and pull requests and do not filter by open or closed state",
            );

        let issues = Arg::with_name("issues")
            .long("issues")
            .short("i")
            .help("List issues")
            .long_help("List issues. This is assumed true by default unless -p or -r is given, in which case this flag must explicitly be given in order to include issues.");

        let pull_requests = Arg::with_name("pull requests")
            .long("pull-requests")
            .short("p")
            .help("List pull requests");

        let review_requests = Arg::with_name("review requests")
            .long("review-requests")
            .short("r")
            .help("Include requests for review")
            .long_help("List pull requests for which a review has been requested");

        let args: ArgMatches = App::new(crate_name!())
            .about("Command line tool for listing and creating GitHub issues")
            .version(crate_version!())
            .author(crate_authors!())
            .arg(action)
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
            .get_matches();

        args
    }

    pub fn read_repo_from_file() -> String {
        let current_path: &Path = Path::new(".");
        let repo_root: PathBuf = traverse(&current_path);
        let config_file: PathBuf = repo_root.join(".git").join("config");
        log::debug!("Using Git config file: '{:?}'", config_file);
        let file_content: String =
            fs::read_to_string(config_file).expect("Could not find a git config");

        let lines: Vec<&str> = file_content
            .lines()
            .filter(|f| f.contains("github.com"))
            .collect();

        let repo: &str = lines
            .first()
            .expect("No Github repository found")
            .split_terminator(":")
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
}
