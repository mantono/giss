pub mod issue {

    use crate::github_resources::ghrs;
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
        #[serde(alias = "updatedAt")]
        pub updated_at: String,
        pub state: ghrs::State,
        pub comments: Comments,
        pub assignees: AssigneeNode,
        pub labels: LabelNode,
        pub repository: Repository,
    }

    #[derive(Debug, Deserialize)]
    pub struct IssueV3 {
        pub url: String,
        pub id: u64,
        pub number: u32,
        pub title: String,
        pub body: Option<String>,
        pub updated_at: String,
        pub state: ghrs::State,
        pub comments: u32,
        pub assignees: Vec<Assignee>,
        pub pull_request: Option<PullRequestV3>,
        pub labels: Vec<Label>,
    }

    impl IssueV3 {
        pub fn is_pull_request(&self) -> bool {
            match self.pull_request {
                Some(_) => true,
                None => false,
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct PullRequestV3 {
        pub url: String,
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
    pub struct Assignee {
        pub login: String,
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
    pub struct AssigneeNode {
        pub nodes: Vec<Assignee>,
    }

    impl ghrs::Closeable for Issue {
        fn is_open(&self) -> bool {
            match self.state {
                ghrs::State::Open => true,
                ghrs::State::Closed => false,
            }
        }
    }
}
