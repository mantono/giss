use crate::{github_resources::ghrs, search::Type};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Root {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub search: Search,
}

#[derive(Debug, Deserialize)]
pub struct Search {
    pub edges: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub node: Issue,
}

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub url: String,
    #[serde(alias = "databaseId")]
    pub id: u64,
    pub number: u32,
    pub title: String,
    #[serde(alias = "bodyText")]
    pub body: Option<String>,
    #[serde(alias = "createdAt")]
    pub created_at: String,
    #[serde(alias = "updatedAt")]
    pub updated_at: String,
    #[serde(alias = "issueState")]
    #[serde(alias = "pullRequestState")]
    pub state: ghrs::State,
    pub comments: Comments,
    pub reactions: Reactions,
    pub assignees: AssigneeNode,
    #[serde(alias = "reviewRequests")]
    pub review_requets: Option<ReviewRequestNode>,
    pub labels: LabelNode,
    pub repository: Repository,
    #[serde(alias = "__typename")]
    pub kind: Type,
}

impl PartialEq for Issue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Issue {
    pub fn has_review_request(&self, user: &str) -> bool {
        match &self.review_requets {
            Some(req) => {
                let users: Vec<&String> = req.nodes.iter().map(|n| &n.requested_reviewer.login).collect();
                users.contains(&&user.to_string())
            }
            None => false,
        }
    }

    pub fn link(&self) -> String {
        let repo: &String = &self.repository.name_with_owner;
        let kind: &str = match self.kind {
            Type::Issue => "issues",
            Type::PullRequest | Type::ReviewRequest => "pull",
        };
        let number: u32 = self.number;
        format!("https://github.com/{}/{}/{}", repo, kind, number)
    }
}

#[derive(Debug, Deserialize)]
pub struct LabelNode {
    pub nodes: Vec<Label>,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UserFields {
    pub login: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(alias = "nameWithOwner")]
    pub name_with_owner: String,
}

#[derive(Debug, Deserialize)]
pub struct Comments {
    #[serde(alias = "totalCount")]
    pub total_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct Reactions {
    #[serde(alias = "totalCount")]
    pub total_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct AssigneeNode {
    pub nodes: Vec<UserFields>,
}
#[derive(Debug, Deserialize)]
pub struct ReviewRequestNode {
    pub nodes: Vec<RequestedReviewer>,
}

#[derive(Debug, Deserialize)]
pub struct RequestedReviewer {
    #[serde(alias = "requestedReviewer")]
    pub requested_reviewer: UserFields,
}

impl ghrs::Closeable for Issue {
    fn is_open(&self) -> bool {
        match self.state {
            ghrs::State::Open => true,
            ghrs::State::Closed => false,
        }
    }
}
