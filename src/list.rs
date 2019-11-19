pub mod list {

    use crate::issue::issue::{Assignee, Issue, IssueRequest, Label};

    pub struct FilterConfig {
        assigned_only: bool,
        pull_requests: bool,
        review_requests: bool,
    }

    impl FilterConfig {
        pub fn from_args(args: &clap::ArgMatches) -> FilterConfig {
            FilterConfig {
                assigned_only: args.is_present("assigned"),
                pull_requests: args.is_present("pull requests"),
                review_requests: args.is_present("review requests"),
            }
        }
    }

    pub fn list_issues(targets: &Vec<&str>, token: &String, config: &FilterConfig) {
        match targets.len() {
            0 => panic!("No target found"),
            1 => {
                list_issues_for_target(targets.first().expect("Must be one target"), token, config)
            }
            _ => list_issues_for_targets(targets, token, config),
        }
    }

    fn list_issues_for_target(target: &str, token: &String, config: &FilterConfig) {
        if target.contains("/") {
            list_issues_repo(target, token, config)
        } else {
            list_issues_org(target, token, config)
        }
    }

    fn list_issues_repo(repo: &str, token: &String, config: &FilterConfig) {
        let url: String = [crate::GITHUB_API, "repos", repo, "issues"].join("/");
        let client = reqwest::Client::new();
        let mut response: reqwest::Response = client
            .get(&url)
            .bearer_auth(token)
            .send()
            .expect("Request to Github API failed");
        let issues: Vec<Issue> = response.json().expect("Unable to process body in response");
        issues
            .iter()
            .filter(|i| i.state == "open")
            .for_each(print_issue);
    }
    fn list_issues_org(org: &str, token: &String, config: &FilterConfig) {
        // Retrieve all issues for organization here
    }
    fn list_issues_for_targets(targets: &Vec<&str>, token: &String, config: &FilterConfig) {
        // Resolve targets, for example filter out individual repo if parent organization is presnet
    }

    fn print_issue(issue: &Issue) {
        let title: String = truncate(issue.title.clone(), 50);
        let assignees: String = issue
            .assignees
            .iter()
            .map(|a: &Assignee| &a.login)
            .map(|s: &String| format!("{}{}", "@", s))
            .collect::<Vec<String>>()
            .join(", ");

        let labels: String = issue
            .labels
            .iter()
            .map(|l: &Label| &l.name)
            .map(|s: &String| format!("{}{}", "#", s))
            .collect::<Vec<String>>()
            .join(", ");

        let extra: String = vec![title, assignees, labels]
            .iter()
            .filter(|i| !i.is_empty())
            .map(|s| s.clone())
            .collect::<Vec<String>>()
            .join(" | ");

        println!("#{} {}", issue.number, extra);
    }

    fn truncate(string: String, max_length: usize) -> String {
        let new_length: usize = std::cmp::min(string.len(), max_length);
        if new_length < string.len() {
            string[..new_length].to_string()
        } else {
            string
        }
    }
}