use crate::issue::{Assignee, Issue, Label, Root};
use crate::search::{GraphQLQuery, SearchIssues, SearchQuery, Sorting, Type};
use crate::Target;
use core::fmt;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

pub fn list_issues(user: &str, targets: &[Target], token: &str, config: &FilterConfig, use_colors: bool) {
    let query: SearchIssues = SearchIssues {
        archived: false,
        assignee: if config.assigned_only {
            Some(user.to_string())
        } else {
            None
        },
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
        review_requested: if config.review_requests {
            Some(user.to_string())
        } else {
            None
        },
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

    for issue in issues {
        print_issue(&issue, true, use_colors)
    }
}

fn print_issue(issue: &Issue, print_repo: bool, use_colors: bool) {
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

    let color_choice = match use_colors {
        true => ColorChoice::Always,
        false => ColorChoice::Never,
    };

    let mut stdout = StandardStream::stdout(color_choice);

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
