pub mod args {
    use clap::{App, Arg, ArgMatches};
    use regex::Regex;
    use std::fs;

    pub fn parse_args() -> ArgMatches<'static> {
        let action = Arg::with_name("action")
            .default_value("list")
            .help("Action to take")
            .long_help("Choose whether to list issues or create a new one.")
            .possible_values(&["list", "create"]);

        //let current_repo: String = read_repo_from_file();
        let target_regex = Regex::new(r"^[\w\-]+(/[\w\-_\.]+)?$").unwrap();
        let target = Arg::with_name("target")
        .takes_value(true)
        .multiple(true)
        .help("Name of target(s)")
        .long_help("Name of the targets for the action. Can be one or multiple organizations, owners or repositories. Any repository specified must be qualified with the owner or organization name. For example 'org/repo0 org/repo1 other-org'. If action is 'create' then only one target will be accepted. When no target is specified, repositorty in current directory will be used, if possible.")
        .validator(move |i| {
            if target_regex.is_match(&i) {
                Ok(())
            } else {
                Err(format!("Invalid target pattern: '{}'", i))
            }
        })
        .default_value(&read_repo_from_file());

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
            .help("Filter issues assigned to user")
            .long_help("Only include issues assigned to user");

        let state = Arg::with_name("state")
        .long("state")
        .short("s")
        .takes_value(true)
        .default_value("open")
        .possible_values(&["open", "closed", "all"])
        .help("Filter issues by state")
        .long_help("Filter issues and pull request on whether they are in state open or closed, or choose to include all regardless of current state.");

        let pull_requests = Arg::with_name("pull requests")
            .long("pull-requests")
            .short("p")
            .help("Include assigned pull requests")
            .long_help("List pull requests in addition to issues.");

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
            .arg(state)
            .arg(pull_requests)
            .arg(review_requests)
            .get_matches();

        args
    }

    fn read_repo_from_file() -> String {
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
