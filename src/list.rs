use crate::search::{GraphQLQuery, SearchIssues, SearchQuery, Type};
use crate::{
    api::ApiError,
    cfg::Config,
    issue::{Issue, Root},
    project::Project,
    sort::Sorting,
    AppErr,
};
use crate::{user::Username, Target};
use core::fmt;
use std::{
    sync::mpsc::{SendError, SyncSender},
    time::Instant,
};

#[derive(Debug)]
pub struct FilterConfig {
    assigned_only: bool,
    pull_requests: bool,
    review_requests: bool,
    issues: bool,
    labels: Vec<String>,
    project: Option<Project>,
    sorting: Sorting,
    search: Option<String>,
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
            labels: cfg.label(),
            project: cfg.project(),
            sorting: cfg.sorting(),
            search: cfg.search(),
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

pub async fn list_issues(
    channel: SyncSender<Issue>,
    user: &Option<Username>,
    targets: &[Target],
    token: &str,
    config: &FilterConfig,
) -> Result<(), AppErr> {
    let user: Option<String> = user.clone().map(|u| u.0);
    log::debug!("Filter config: {:?}", config);

    let start = Instant::now();

    let one = async {
        if config.issues {
            req_and_send(Type::Issue, &channel, &user, targets, token, config).await?;
        }
        Ok::<(), AppErr>(())
    };

    let two = async {
        if config.pull_requests {
            req_and_send(Type::PullRequest, &channel, &user, targets, token, config).await?;
        }
        Ok::<(), AppErr>(())
    };

    let three = async {
        if config.review_requests {
            req_and_send(Type::ReviewRequest, &channel, &user, targets, token, config).await?;
        }
        Ok::<(), AppErr>(())
    };

    futures::try_join!(one, two, three)?;

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::debug!("API execution took {:?}", elapsed);

    Ok(())
}

async fn req_and_send(
    kind: Type,
    channel: &SyncSender<Issue>,
    user: &Option<String>,
    targets: &[Target],
    token: &str,
    config: &FilterConfig,
) -> Result<(), AppErr> {
    let query: SearchIssues = create_query(kind, &user, targets, config);
    let issues: Vec<Issue> = api_request(query, token).await?;

    for issue in issues {
        channel.send(issue)?;
    }

    Ok(())
}

fn create_query(kind: Type, user: &Option<String>, targets: &[Target], config: &FilterConfig) -> SearchIssues {
    let assignee: Option<String> = match config.assigned_only {
        false => None,
        true => match kind {
            Type::Issue | Type::PullRequest => user.clone(),
            Type::ReviewRequest => None,
        },
    };

    let review_requested: Option<String> = match config.review_requests {
        false => None,
        true => match kind {
            Type::ReviewRequest => user.clone(),
            Type::Issue | Type::PullRequest => None,
        },
    };
    SearchIssues {
        archived: false,
        assignee,
        resource_type: Some(kind),
        review_requested,
        sort: config.sorting,
        state: config.state,
        labels: config.labels.clone(),
        project: config.project.clone(),
        targets: targets.to_vec(),
        search: config.search.clone(),
        limit: config.limit,
    }
}

impl From<SendError<Issue>> for AppErr {
    fn from(_: SendError<Issue>) -> Self {
        AppErr::ChannelError
    }
}

impl From<ApiError> for AppErr {
    fn from(err: ApiError) -> Self {
        log::error!("{:?}", err);
        match err {
            ApiError::NoResponse(_) => AppErr::ApiError,
            ApiError::Response(code) => match code {
                429 => AppErr::RateLimited,
                _ => AppErr::ApiError,
            },
        }
    }
}

async fn api_request(search: SearchIssues, token: &str) -> Result<Vec<Issue>, ApiError> {
    let query: GraphQLQuery = search.build();
    let issues: Root = crate::api::v4::request(token, query).await?;
    let issues: Vec<Issue> = issues.data.search.edges.into_iter().map(|n| n.node).collect();
    Ok(issues)
}
