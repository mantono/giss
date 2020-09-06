use crate::issue::{Assignee, IdNode, Issue, IssueV3, Label, Root, UserId};
use crate::search::{GraphQLQuery, SearchIssues, SearchQuery, Sorting, Type};
use crate::user;
use crate::Target;
use core::fmt;
use itertools::Itertools;

#[derive(Debug)]
pub struct FilterConfig {
    assigned_only: bool,
    pull_requests: bool,
    review_requests: bool,
    issues: bool,
    state: FilterState,
    limit: u32,
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
        let assigned_only: bool = args.is_present("assigned");

        let pull_requests: bool = args.is_present("pull requests");
        let review_requests: bool = args.is_present("review requests");
        let issues: bool = args.is_present("issues");

        let filter_all: bool = !pull_requests && !review_requests && !issues;

        let pull_requests: bool = pull_requests || filter_all;
        let review_requests: bool = review_requests || filter_all;
        let issues: bool = issues || filter_all;

        let limit: u32 = args.value_of("limit").unwrap().parse().expect("Invalid number");

        FilterConfig {
            assigned_only,
            pull_requests,
            review_requests,
            issues,
            state,
            limit,
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

pub fn list_issues(user: &str, targets: &Vec<Target>, token: &str, config: &FilterConfig) {
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

fn list_issues_targets(user: &str, target: &Vec<Target>, token: &str, config: &FilterConfig) {
    let orgs: Vec<String> = target.iter().filter_map(|t| t.as_org()).collect();
    list_issues_orgs(user, &orgs, token, config)
}

fn list_issues_repo(org: &str, repo: &str, token: &str, config: &FilterConfig) {
    let mut url: String = [crate::GITHUB_API_V3_URL, "repos", org, repo, "issues?"].join("/");

    let mut query_parameters: Vec<(String, String)> = vec![
        ("state".to_string(), config.state.to_string()),
        ("sort".to_string(), "updated".to_string()),
        ("direction".to_string(), "desc".to_string()),
    ];

    if config.assigned_only {
        query_parameters.push(("assignee".to_string(), user::fetch_username(token)))
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

    let status_code: &u16 = &response.status().as_u16();
    match status_code {
        400u16..=599u16 => log::error!(
            "GitHub API response: {} - {}",
            status_code,
            response.text().unwrap_or_default()
        ),
        _ => log::debug!("GitHub API response: {}", status_code),
    }

    let issues: Vec<IssueV3> = response.json().expect("Unable to process body in response");
    issues
        .iter()
        .filter(|i| filter_issue(&i, &config))
        .take(config.limit as usize)
        .for_each(|i| print_issue_v3(&i));
}

fn filter_issue(issue: &IssueV3, config: &FilterConfig) -> bool {
    let allow_issue: bool = config.issues && !issue.is_pull_request();
    let filter_prs: bool = config.pull_requests || config.review_requests;
    let allow_pr: bool = filter_prs && issue.is_pull_request();
    allow_issue || allow_pr
}

const GITHUB_API_V4_URL: &str = "https://api.github.com/graphql";

fn list_issues_orgs(user: &str, targets: &Vec<String>, token: &str, config: &FilterConfig) {
    let query: SearchIssues = SearchIssues {
        archived: false,
        assignee: if config.assigned_only {
            Some(user.to_string())
        } else {
            None
        },
        resource_type: match (config.issues, config.pull_requests, config.review_requests) {
            (true, false, false) => Some(Type::Issue),
            (true, _, _) => None,
            (false, _, _) => Some(Type::PullRequest),
        },
        review_requested: if config.review_requests {
            Some(user.to_string())
        } else {
            None
        },
        sort: (String::from("updated"), Sorting::Descending),
        state: config.state,
        users: targets.clone(),
        limit: config.limit,
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

    let mut response: reqwest::Response = client.execute(request).expect("Request failed to GitHub v4 API");

    let issues: Root = response.json().expect("Unable to parse body as JSON");
    let user_id: String = issues.data.viewer.id;

    let issues: Vec<&Issue> = issues
        .data
        .search
        .edges
        .iter()
        .map(|n| &n.node)
        .filter(|i: &&Issue| !config.review_requests || i.user_has_review_req(&user_id))
        .collect();

    for issue in issues {
        print_issue(&issue, true)
    }
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
