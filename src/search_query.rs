pub mod search {
    use crate::list::list::FilterState;
    use itertools::Itertools;
    use std::fmt;

    enum Sorting {
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

    trait SearchQuery {
        fn search_type(&self) -> Option<String>;
        fn build(&self) -> String;
    }

    struct SearchIssues {
        state: FilterState,
        assignee: Option<String>,
        archived: bool,
        users: Vec<String>,
        sort: (String, Sorting),
    }

    impl SearchQuery for SearchIssues {
        fn search_type(&self) -> Option<String> {
            Some(String::from("type:issue"))
        }

        fn build(&self) -> String {
            let parts: Vec<String> = [
                self.search_type(),
                self.state(),
                self.assignee(),
                self.archived(),
                self.users(),
                self.state(),
            ]
            .iter()
            .filter_map(|v| v.clone())
            .collect();

            parts.join(" ")
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

    struct SearchPullRequests {
        state: FilterState,
        assignee: Option<String>,
        review_requested: Option<String>,
        archived: bool,
        users: Vec<String>,
        sort: (String, Sorting),
    }

    impl SearchQuery for SearchPullRequests {
        fn search_type(&self) -> Option<String> {
            Some(String::from("type:pr"))
        }

        fn build(&self) -> String {
            let parts: Vec<String> = [
                self.search_type(),
                self.state(),
                self.assignee(),
                self.archived(),
                self.users(),
                self.state(),
            ]
            .iter()
            .filter_map(|v| v.clone())
            .collect();

            parts.join(" ")
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
