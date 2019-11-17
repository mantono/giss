pub mod issue {

    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Deserialize)]
    pub struct Issue {
        pub url: String,
        pub id: u64,
        pub number: u32,
        pub title: String,
        pub body: Option<String>,
        pub updated_at: String,
        pub state: String,
        pub comments: u32,
        pub assignees: Vec<Assignee>,
        pub labels: Vec<Label>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Label {
        pub name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Assignee {
        pub login: String,
    }

    #[derive(Debug, Serialize)]
    pub struct IssueRequest {
        pub title: String,
        pub body: Option<String>,
        pub labels: Vec<String>,
        pub assignees: Vec<String>,
    }
}
