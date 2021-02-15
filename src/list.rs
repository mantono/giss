use crate::search::{GraphQLQuery, SearchIssues, SearchQuery, Sorting, Type};
use crate::{
    cfg::Config,
    issue::{Assignee, Issue, Label, Root},
};
use crate::{user::Username, Target};
use core::fmt;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
pub struct FilterConfig {
    assigned_only: bool,
    pull_requests: bool,
    review_requests: bool,
    issues: bool,
    state: StateFilter,
    limit: u32,
}

impl From<Config> for FilterConfig {
    fn from(cfg: Config) -> Self {
        FilterConfig {
            assigned_only: cfg.assigned_only(),
            pull_requests: cfg.pulls(),
            review_requests: cfg.reviews(),
            issues: cfg.issues(),
            state: cfg.state(),
            limit: cfg.limit(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum StateFilter {
    Open,
    Closed,
    All,
}

impl std::fmt::Display for StateFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            StateFilter::Open => "open",
            StateFilter::Closed => "closed",
            StateFilter::All => "all",
        };
        write!(f, "{}", output)
    }
}

pub fn list_issues(
    user: &Option<Username>,
    targets: &[Target],
    token: &str,
    config: &FilterConfig,
    use_colors: ColorChoice,
) {
    let user: Option<String> = user.clone().map(|u| u.0);
    let query: SearchIssues = SearchIssues {
        archived: false,
        assignee: if config.assigned_only { user.clone() } else { None },
        resource_type: match (config.issues, config.pull_requests, config.review_requests) {
            (true, false, false) => Some(Type::Issue),
            (false, true, false) => Some(Type::PullRequest),
            (false, false, true) => Some(Type::ReviewRequest),
            (false, false, false) => None,
            (_, _, _) => panic!(
                "Illegal combination: {}, {}, {}",
                config.issues, config.pull_requests, config.review_requests
            ),
        },
        review_requested: if config.review_requests { user.clone() } else { None },
        sort: (String::from("updated"), Sorting::Descending),
        state: config.state,
        targets: targets.to_vec(),
        limit: config.limit,
    };
    let query: GraphQLQuery = query.build();

    let issues: Root = match crate::api::v4::request(token, query) {
        Ok(body) => body,
        Err(e) => {
            log::error!("Error, status code: {}", e);
            std::process::exit(4)
        }
    };

    let issues: Vec<&Issue> = issues.data.search.edges.iter().map(|n| &n.node).collect();

    let print_repo: bool = match (targets.len(), targets.first().unwrap()) {
        (1, Target::Repository { .. }) => false,
        _ => true,
    };

    for issue in issues {
        print_issue(&issue, print_repo, use_colors)
    }
}

fn print_issue(issue: &Issue, print_repo: bool, use_colors: ColorChoice) {
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

    let mut stdout = StandardStream::stdout(use_colors);

    if print_repo {
        write!(&mut stdout, "#{} {}", issue.number, repo).unwrap();
    } else {
        write!(&mut stdout, "#{}", issue.number).unwrap();
    }

    write(&mut stdout, " | ", Some(Color::Green));
    write(&mut stdout, &title, None);

    if !assignees.is_empty() {
        write(&mut stdout, " | ", Some(Color::Green));
        write(&mut stdout, &assignees, Some(Color::Cyan));
    }

    if !labels.is_empty() {
        write(&mut stdout, " | ", Some(Color::Green));
        write(&mut stdout, &labels, Some(Color::Magenta));
    }

    write(&mut stdout, "\n", None);
}

fn truncate(string: String, max_length: usize) -> String {
    let new_length: usize = std::cmp::min(string.len(), max_length);
    if new_length < string.len() {
        string[..new_length].to_string()
    } else {
        string
    }
}

fn write(stream: &mut StandardStream, content: &str, color: Option<Color>) {
    stream.set_color(ColorSpec::new().set_fg(color)).unwrap();
    write!(stream, "{}", content).unwrap();
}
