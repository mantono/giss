pub mod list {

    use crate::github_resources::ghrs;
    use crate::issue::issue::{Assignee, Data, Issue, IssueRequest, IssueV3, Label, Root};
    use crate::search_query::search::{GraphQLQuery, SearchIssues, SearchQuery, Sorting, Type};
    use crate::user::usr;
    use crate::Target;
    use core::fmt;
    use itertools::Itertools;
    use log::Level;
    use serde::private::ser::constrain;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::error::Error;
    use std::panic::resume_unwind;

    pub struct FilterConfig {
        assigned_only: bool,
        pull_requests: bool,
        review_requests: bool,
        issues: bool,
        state: FilterState,
    }

    impl FilterConfig {
        pub fn from_args(args: &clap::ArgMatches) -> FilterConfig {
            let state: FilterState = if args.is_present("closed") {
                FilterState::Closed
            } else if args.is_present("all") {
                FilterState::All
            } else {
                FilterState::Open
            };
            let pull_requests: bool = args.is_present("pull requests");
            let review_requests: bool = args.is_present("review requests");
            let issues: bool = args.is_present("issues") || (!pull_requests && !review_requests);
            FilterConfig {
                assigned_only: args.is_present("assigned"),
                pull_requests,
                review_requests,
                issues,
                state,
            }
        }
    }

    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    pub enum FilterState {
        Open,
        Closed,
        All,
    }

    impl std::fmt::Display for FilterState {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let output = match self {
                FilterState::Open => "open",
                FilterState::Closed => "closed",
                FilterState::All => "all",
            };
            write!(f, "{}", output)
        }
    }

    pub fn list_issues(
        user: &String,
        targets: &Vec<Target>,
        token: &String,
        config: &FilterConfig,
    ) {
        match targets.len() {
            0 => panic!("No target found"),
            1 => {
                let target: &Target = targets.first().expect("Must be one target");
                if let Target::Repository { name, owner } = target {
                    list_issues_repo(owner, name, token, config)
                } else {
                    list_issues_targets(user, targets, token, config)
                }
            }
            _ => list_issues_targets(user, targets, token, config),
        }
    }

    fn list_issues_targets(
        user: &String,
        target: &Vec<Target>,
        token: &String,
        config: &FilterConfig,
    ) {
        let orgs: Vec<String> = target.iter().filter_map(|t| t.as_org()).collect();
        list_issues_orgs(user, &orgs, token, config)
    }

    fn list_issues_repo(org: &String, repo: &String, token: &String, config: &FilterConfig) {
        let mut url: String = [crate::GITHUB_API_V3_URL, "repos", org, repo, "issues?"].join("/");

        let mut query_parameters: Vec<(String, String)> = vec![
            ("state".to_string(), config.state.to_string()),
            ("sort".to_string(), "updated".to_string()),
            ("direction".to_string(), "desc".to_string()),
        ];

        if config.assigned_only {
            query_parameters.push(("assignee".to_string(), usr::fetch_username(token)))
        }

        let query_parameters: String = query_parameters
            .iter()
            .map(|(k, v)| {
                let mut k = k.clone();
                k.push_str("=");
                k.push_str(v);
                k
            })
            .join("&");

        url.extend(query_parameters.chars());
        log::debug!("{:?}", url);
        let client = reqwest::Client::new();
        let mut response: reqwest::Response = client
            .get(&url)
            .bearer_auth(token)
            .send()
            .expect("Request to Github API failed");
        let issues: Vec<IssueV3> = response.json().expect("Unable to process body in response");
        issues.iter().for_each(|i| print_issue_v3(&i));
    }

    const GITHUB_API_V4_URL: &str = "https://api.github.com/graphql";

    fn list_issues_orgs(
        user: &String,
        targets: &Vec<String>,
        token: &String,
        config: &FilterConfig,
    ) {
        let query: SearchIssues = SearchIssues {
            archived: false,
            assignee: if config.assigned_only {
                Some(user.clone())
            } else {
                None
            },
            resource_type: match (config.issues, config.pull_requests, config.review_requests) {
                (true, false, false) => Some(Type::Issue),
                (true, _, _) => None,
                (false, _, _) => Some(Type::PullRequest),
            },
            review_requested: if config.review_requests {
                Some(user.clone())
            } else {
                None
            },
            sort: (String::from("updated"), Sorting::Descending),
            state: config.state.clone(),
            users: targets.clone(),
        };
        let query: GraphQLQuery = query.build();
        log::debug!("{}", query.variables);
        let client = reqwest::Client::new();
        let request: reqwest::Request = client
            .post(GITHUB_API_V4_URL)
            .bearer_auth(token)
            .json(&query)
            .build()
            .expect("Failed to build query");

        let mut response: reqwest::Response = client
            .execute(request)
            .expect("Request failed to GitHub v4 API");

        let issues: Root = response.json().expect("Unable to parse body as JSON");
        print_issues(issues, true);
    }

    fn print_issues(root: Root, print_repo: bool) {
        for node in root.data.search.edges {
            print_issue(&node.node, print_repo)
        }
    }

    fn print_issue(issue: &Issue, print_repo: bool) {
        let title: String = truncate(issue.title.clone(), 50);
        let assignees: String = issue
            .assignees
            .nodes
            .iter()
            .map(|a: &Assignee| &a.login)
            .map(|s: &String| format!("{}{}", "@", s))
            .collect::<Vec<String>>()
            .join(", ");

        let repo: String = if print_repo {
            issue.repository.name_with_owner.clone()
        } else {
            String::from("")
        };

        let labels: String = issue
            .labels
            .nodes
            .iter()
            .map(|l: &Label| &l.name)
            .map(|s: &String| format!("{}{}", "#", s))
            .collect::<Vec<String>>()
            .join(", ");

        let extra: String = vec![repo, title, assignees, labels]
            .iter()
            .filter(|i| !i.is_empty())
            .map(|s| s.clone())
            .collect::<Vec<String>>()
            .join(" | ");

        println!("#{} {}", issue.number, extra);
    }

    fn print_issue_v3(issue: &IssueV3) {
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
