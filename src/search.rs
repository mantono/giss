use crate::{list::StateFilter, project::Project};
use crate::{sort::Sorting, Target};
use itertools::Itertools;
use serde::Serialize;
use serde_json::json;
use std::fmt;

#[derive(Serialize, Debug)]
pub struct GraphQLQuery {
    pub query: String,
    pub variables: serde_json::Value,
    pub operation_name: String,
}

pub trait SearchQuery {
    fn search_type(&self) -> Option<String>;
    fn build(&self) -> GraphQLQuery;
}

pub enum Type {
    Issue,
    PullRequest,
    ReviewRequest,
}

pub struct SearchIssues {
    pub state: StateFilter,
    pub assignee: Option<String>,
    pub review_requested: Option<String>,
    pub archived: bool,
    pub labels: Vec<String>,
    pub project: Option<Project>,
    pub resource_type: Option<Type>,
    pub targets: Vec<Target>,
    pub sort: Sorting,
    pub limit: u32,
}

impl SearchQuery for SearchIssues {
    fn search_type(&self) -> Option<String> {
        match self.resource_type {
            Some(Type::Issue) => Some(String::from("type:issue")),
            Some(Type::PullRequest) => Some(String::from("type:pr")),
            Some(Type::ReviewRequest) => {
                let reviewer = self.review_requested.as_ref().expect("Reviewer was not sent");
                let query: String = format!("type:pr review-requested:{}", reviewer);
                Some(query)
            }
            None => None,
        }
    }

    fn build(&self) -> GraphQLQuery {
        let parts: Vec<String> = [
            self.search_type(),
            self.state(),
            self.assignee(),
            self.archived(),
            self.users(),
            self.labels(),
            self.project(),
            self.sort(),
        ]
        .iter()
        .filter_map(|v| v.clone())
        .collect();

        let search_query: String = parts.join(" ");

        log::debug!("Search query: '{}'", search_query);
        GraphQLQuery {
            variables: json!({
                "searchQuery": search_query,
                "limit": self.limit
            }),
            query: String::from(include_str!("../data/graphql/queries/search_issues.graphql")),
            operation_name: String::from("SearchIssues"),
        }
    }
}

impl SearchIssues {
    fn state(&self) -> Option<String> {
        match self.state {
            StateFilter::All => None,
            StateFilter::Open => Some(String::from("state:open")),
            StateFilter::Closed => Some(String::from("state:closed")),
        }
    }

    fn assignee(&self) -> Option<String> {
        match &self.assignee {
            Some(name) => Some(format!("assignee:{}", name)),
            None => None,
        }
    }

    fn archived(&self) -> Option<String> {
        Some(String::from("archived:false"))
    }

    fn users(&self) -> Option<String> {
        if self.targets.is_empty() {
            None
        } else {
            let users: String = self.targets.iter().map(|user| user.to_string()).join(" ");
            Some(users)
        }
    }

    fn labels(&self) -> Option<String> {
        if self.labels.is_empty() {
            None
        } else {
            Some(self.labels.iter().map(|l| format!("label:{}", l)).join(" "))
        }
    }

    fn project(&self) -> Option<String> {
        self.project.clone().map(|p| format!("project:{}", p))
    }

    fn sort(&self) -> Option<String> {
        Some(format!("sort:{}", self.sort))
    }
}
