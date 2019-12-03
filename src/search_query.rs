pub mod search {
    use crate::list::list::FilterState;
    use itertools::Itertools;
    use serde::Serialize;
    use serde_json::json;
    use serde_json::Value;
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

    pub struct SearchIssues {
        pub state: FilterState,
        pub assignee: Option<String>,
        pub archived: bool,
        pub users: Vec<String>,
        pub sort: (String, Sorting),
    }

    impl SearchQuery for SearchIssues {
        fn search_type(&self) -> Option<String> {
            Some(String::from("type:issue"))
        }

        fn build(&self) -> GraphQLQuery {
            let parts: Vec<String> = [
                self.search_type(),
                self.state(),
                self.assignee(),
                self.archived(),
                self.users(),
                self.state(),
                self.sort(),
            ]
            .iter()
            .filter_map(|v| v.clone())
            .collect();

            let search_query: String = parts.join(" ");

            GraphQLQuery {
                variables: json!({
                    "searchQuery": search_query,
                    "limit": 10
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
            Some(String::from("updatedAt-desc"))
        }
    }

    pub struct SearchPullRequests {
        pub state: FilterState,
        pub assignee: Option<String>,
        pub review_requested: Option<String>,
        pub archived: bool,
        pub users: Vec<String>,
        pub sort: (String, Sorting),
    }

    impl SearchQuery for SearchPullRequests {
        fn search_type(&self) -> Option<String> {
            Some(String::from("type:pr"))
        }

        fn build(&self) -> GraphQLQuery {
            let parts: Vec<String> = [
                self.search_type(),
                self.state(),
                self.assignee(),
                self.archived(),
                self.users(),
                self.state(),
                self.sort(),
            ]
            .iter()
            .filter_map(|v| v.clone())
            .collect();

            let search_query: String = parts.join(" ");

            GraphQLQuery {
                variables: json!({
                    "searchQuery": search_query,
                    "limit": 10
                }),
                query: String::from(include_str!(
                    "../data/graphql/queries/search_pull_requests.graphql"
                )),
                operation_name: String::from("SearchPullRequests"),
            }
        }
    }

    impl SearchPullRequests {
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
                None => match &self.review_requested {
                    Some(name) => Some(format!("review_requested:{}", name)),
                    None => None,
                },
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
            Some(String::from("updatedAt-desc"))
        }
    }
}

/*
{
    "searchQuery": "is:open assignee:mantono archived:false user:zensum user:klira user:open-broker sort:comments-desc"
}
*/
