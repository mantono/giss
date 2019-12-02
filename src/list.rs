pub mod list {

    use crate::github_resources::ghrs;
    use crate::issue::issue::{Assignee, Issue, IssueRequest, Label};
    use crate::Target;
    use core::fmt;
    use graphql_client::{GraphQLQuery, QueryBody, Response};
    use itertools::{all, Itertools};
    use serde::private::ser::constrain;
    use serde::Deserialize;
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::ptr::hash;

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

    #[derive(Eq, PartialEq, Debug)]
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

    pub fn list_issues(targets: &Vec<Target>, token: &String, config: &FilterConfig) {
        match targets.len() {
            0 => panic!("No target found"),
            1 => {
                let target: &Target = targets.first().expect("Must be one target");
                if let Target::Repository { name, owner } = target {
                    list_issues_repo(owner, name, token, config)
                } else {
                    list_issues_targets(targets, token, config)
                }
            }
            _ => list_issues_targets(targets, token, config),
        }
    }

    fn list_issues_targets(target: &Vec<Target>, token: &String, config: &FilterConfig) {
        let orgs: Vec<String> = target.iter().filter_map(|t| t.as_org()).collect();
        list_issues_orgs(&orgs, token, config)
    }

    #[derive(GraphQLQuery, Debug)]
    #[graphql(
        schema_path = "data/graphql/schema/schema.public.graphql",
        query_path = "data/graphql/queries/search_issues.graphql"
    )]
    struct SearchIssues;

    #[derive(GraphQLQuery, Debug)]
    #[graphql(
        schema_path = "data/graphql/schema/schema.public.graphql",
        query_path = "data/graphql/queries/search_pull_requests.graphql",
        response_derives = "Debug"
    )]
    struct SearchPullRequests;

    #[derive(Deserialize, Debug)]
    pub struct DateTime {
        timestamp: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct URI {
        uri: String,
    }

    #[derive(Deserialize)]
    struct User {
        login: String,
    }

    fn fetch_username(token: &String) -> String {
        match get_saved_username(token) {
            Some(username) => username,
            None => {
                let username: String = api_lookup_username(token);
                save_username(token, &username).expect("Unable to save username");
                username
            }
        }
    }

    fn save_username(token: &String, username: &String) -> Result<(), std::io::Error> {
        let token_hash: String = hash_token(token);
        let mut path: PathBuf = get_users_dir();
        std::fs::create_dir_all(&path).expect("Unable to create path");
        path.push(token_hash);
        let mut file: File = File::create(&path).expect("Unable to create file");
        file.write_all(username.as_bytes())
    }

    fn api_lookup_username(token: &String) -> String {
        let client = reqwest::Client::new();
        let mut response: reqwest::Response = client
            .get("https://api.github.com/user")
            .bearer_auth(token)
            .send()
            .expect("Request to Github API failed when fetching user name");

        response
            .json::<User>()
            .expect("Unable to parse GitHub user")
            .login
    }

    fn hash_token(token: &String) -> String {
        let mut hasher = Sha256::new();
        hasher.input(token);
        format!("{:02x}", hasher.result().as_slice().iter().format(""))
    }

    fn get_users_dir() -> PathBuf {
        let mut path: PathBuf = dirs::home_dir().expect("Cannot find home dir");
        path.push([".config", "giss", "usernames"].join("/"));
        path
    }

    fn get_saved_username(token: &String) -> Option<String> {
        let token_hash: String = hash_token(token);
        let mut path: PathBuf = get_users_dir();
        path.push(token_hash);
        if path.exists() {
            let content = std::fs::read_to_string(path).expect("Unable to read content of file");
            Some(content)
        } else {
            None
        }
    }

    fn list_issues_repo(org: &String, repo: &String, token: &String, config: &FilterConfig) {
        let mut url: String = [crate::GITHUB_API, "repos", org, repo, "issues?"].join("/");

        let mut query_parameters: Vec<(String, String)> = vec![
            ("state".to_string(), config.state.to_string()),
            ("sort".to_string(), "updated".to_string()),
            ("direction".to_string(), "desc".to_string()),
        ];

        if config.assigned_only {
            query_parameters.push(("assignee".to_string(), fetch_username(token)))
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

    fn list_issues_orgs(targets: &Vec<String>, token: &String, config: &FilterConfig) {
        //        SearchIssues::
        //        let body: &str = include_str!("data/graphql/search_pull_requests.graphql");
        //        let client = reqwest::Client::new();
        //        let mut resposne: reqwest::Response = client
        //            .get(&url)
        //            .bearer_auth(token)
        //            .body()
        let variables = search_issues::Variables {
            search_query: build_search_query_issues(&String::from("mantono"), targets, config),
            limit: Some(10),
        };
        let search: QueryBody<_> = SearchIssues::build_query(variables);
        println!("{:?}", targets);
        let client = reqwest::Client::new();
        let mut response: reqwest::Response = client
            .get(GITHUB_API_V4_URL)
            .bearer_auth(token)
            .json(&search)
            .send()
            .expect("Request failed to GitHub v4 API");

        println!("{:?}", response);
        println!(
            "{}, {}, {:?}",
            search.operation_name, search.query, variables
        );
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
