pub mod list {

    use crate::github_resources::ghrs;
    use crate::issue::issue::{Assignee, Issue, IssueRequest, Label};
    use crate::search_query::search::{GraphQLQuery, SearchIssues, SearchQuery, Sorting};
    use crate::user::usr;
    use crate::Target;
    use core::fmt;
    use itertools::{all, Itertools};
    use serde::private::ser::constrain;
    use serde::Deserialize;
    use serde_json::json;
    use std::collections::HashMap;

    pub struct FilterConfig {
        assigned_only: bool,
        pull_requests: bool,
        review_requests: bool,
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
            FilterConfig {
                assigned_only: args.is_present("assigned"),
                pull_requests: args.is_present("pull queries"),
                review_requests: args.is_present("review queries"),
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
        println!("{:?}", url);
        let client = reqwest::Client::new();
        let mut response: reqwest::Response = client
            .get(&url)
            .bearer_auth(token)
            .send()
            .expect("Request to Github API failed");
        let issues: Vec<Issue> = response.json().expect("Unable to process body in response");
        issues.iter().for_each(print_issue);
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
            sort: (String::from("updatedAt"), Sorting::Descending),
            state: config.state.clone(),
            users: targets.clone(),
        };
        let query: GraphQLQuery = query.build();
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

        println!("{:?}", response.text());
        //        println!(
        //            "{}, {}, {:?}",
        //            query.query, query.operation_name, query.variables
        //        );
    }

    fn build_search_query_issues(
        user: &String,
        targets: &Vec<String>,
        config: &FilterConfig,
    ) -> String {
        let all_targets: String = targets.iter().map(|t| format!("user:{}", t)).join(" ");
        format!(
            "assignee:{} type:issue archived:false {} sort:updatedAt-desc",
            user, all_targets
        )
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
