pub mod search {
    use crate::list::list::FilterState;
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

    pub enum Sorting {
        Descending,
        Ascending,
    }

    impl fmt::Display for Sorting {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let output: &str = match self {
                Sorting::Ascending => "asc",
                Sorting::Descending => "desc",
            };
            write!(f, "{}", output)
        }
    }

    pub trait SearchQuery {
        fn search_type(&self) -> Option<String>;
        fn build(&self) -> GraphQLQuery;
    }

    pub enum Type {
        Issue,
        PullRequest,
    }

    pub struct SearchIssues {
        pub state: FilterState,
        pub assignee: Option<String>,
        pub review_requested: Option<String>,
        pub archived: bool,
        pub resource_type: Option<Type>,
        pub users: Vec<String>,
        pub sort: (String, Sorting),
        pub limit: u32,
    }

    impl SearchQuery for SearchIssues {
        fn search_type(&self) -> Option<String> {
            match self.resource_type {
                Some(Type::Issue) => Some(String::from("type:issue")),
                Some(Type::PullRequest) => Some(String::from("type:pr")),
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
                self.sort(),
            ]
            .iter()
            .filter_map(|v| v.clone())
            .collect();

            let search_query: String = parts.join(" ");

            GraphQLQuery {
                variables: json!({
                    "searchQuery": search_query,
                    "limit": self.limit
                }),
                query: String::from(include_str!(
                    "../data/graphql/queries/search_issues.graphql"
                )),
                operation_name: String::from("SearchIssues"),
            }
        }
    }

    impl SearchIssues {
        fn state(&self) -> Option<String> {
            match self.state {
                FilterState::All => None,
                FilterState::Open => Some(String::from("state:open")),
                FilterState::Closed => Some(String::from("state:closed")),
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
            if self.users.is_empty() {
                None
            } else {
                let users: String = self
                    .users
                    .iter()
                    .map(|user| format!("user:{}", user))
                    .join(" ");
                Some(users)
            }
        }

        fn sort(&self) -> Option<String> {
            Some(String::from("sort:updated-desc"))
        }
    }
}
