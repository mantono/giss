use crate::{
    cfg::Config,
    issue::{Assignee, Issue, Label, Root},
    AppErr,
};
use crate::{
    issue,
    search::{GraphQLQuery, SearchIssues, SearchQuery, Sorting, Type},
};
use crate::{user::Username, Target};
use core::fmt;
use lazy_static::__Deref;
use std::{
    io::Write,
    sync::mpsc::{SendError, Sender},
};
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

impl FilterConfig {
    pub fn types(&self) -> Vec<Type> {
        let mut types: Vec<Type> = Vec::with_capacity(3);
        if self.issues {
            types.push(Type::Issue)
        }
        if self.pull_requests {
            types.push(Type::PullRequest)
        }
        if self.review_requests {
            types.push(Type::ReviewRequest)
        }
        types
    }
}

impl From<&Config> for FilterConfig {
    fn from(cfg: &Config) -> Self {
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
    channel: Sender<Issue>,
    user: &Option<Username>,
    targets: &[Target],
    token: &str,
    config: &FilterConfig,
) -> Result<(), AppErr> {
    let user: Option<String> = user.clone().map(|u| u.0);
    log::debug!("Filter config: {:?}", config);

    for req_type in config.types() {
        let query: SearchIssues = create_query(req_type, &user, targets, config);
        let issues: Vec<Issue> = api_request(query, token)?;

        for issue in issues {
            channel.send(issue)?;
        }
    }

    Ok(())
}

fn create_query(kind: Type, user: &Option<String>, targets: &[Target], config: &FilterConfig) -> SearchIssues {
    SearchIssues {
        archived: false,
        assignee: if config.assigned_only { user.clone() } else { None },
        resource_type: Some(kind),
        review_requested: if config.review_requests { user.clone() } else { None },
        sort: (String::from("updated"), Sorting::Descending),
        state: config.state,
        targets: targets.to_vec(),
        limit: config.limit,
    }
}

impl From<SendError<Issue>> for AppErr {
    fn from(_: SendError<Issue>) -> Self {
        AppErr::ChannelError
    }
}

impl From<u16> for AppErr {
    fn from(status: u16) -> Self {
        match status {
            429 => AppErr::RateLimited,
            _ => AppErr::ApiError,
        }
    }
}

fn api_request(search: SearchIssues, token: &str) -> Result<Vec<Issue>, u16> {
    let query: GraphQLQuery = search.build();
    let issues: Root = crate::api::v4::request(token, query)?;
    let issues: Vec<Issue> = issues.data.search.edges.into_iter().map(|n| n.node).collect();
    Ok(issues)
}
