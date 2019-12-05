pub mod args {
    use clap::{App, Arg, ArgMatches};
    use regex::Regex;
    use std::fs;

    pub fn parse_args<'a>(current_repo: &'a str) -> ArgMatches<'a> {
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
            .help("List issues");

        let pull_requests = Arg::with_name("pull requests")
            .long("pull-requests")
            .short("p")
            .help("List pull requests");

        let review_requests = Arg::with_name("review requests")
            .long("review-requests")
            .short("r")
            .help("Include requests for review")
            .long_help("List requests for review in addition to issues.");

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
        let file_content: String =
            fs::read_to_string(".git/config").expect("Could not find a git config");
        let lines: Vec<&str> = file_content
            .lines()
            .filter(|f| f.contains("github.com"))
            .collect();
        let repo: &str = lines
            .first()
            .expect("No Github repoistory found")
            .split_terminator(":")
            .last()
            .expect("No match");
        repo.trim_end_matches(".git").to_string()
    }
}
