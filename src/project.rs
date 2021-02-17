use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub struct Project {
    owner: String,
    repo: Option<String>,
    id: u32,
}

impl FromStr for Project {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("/").collect();
        match parts.len() {
            2 => {
                let owner: &str = parts.first().expect("Exactly two should be present");
                let id: u32 = parts
                    .last()
                    .expect("Exactly two should be present")
                    .parse::<u32>()
                    .map_err(|_| String::from("id must be a number"))?;

                let project = Project {
                    owner: owner.to_string(),
                    repo: None,
                    id,
                };
                Ok(project)
            }
            3 => {
                let owner: &str = parts.first().expect("Exactly three should be present");
                let repo: &str = parts.get(1).expect("Exactly three should be present");
                let id: u32 = parts
                    .last()
                    .expect("Exactly three should be present")
                    .parse::<u32>()
                    .map_err(|_| String::from("id must be a number"))?;

                let project = Project {
                    owner: owner.to_string(),
                    repo: Some(repo.to_string()),
                    id,
                };
                Ok(project)
            }
            _ => Err(format!(
                "Invalid argument for project '{}', must have format org/repo/number or org/number",
                s
            )),
        }
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.repo.clone() {
            Some(repo) => write!(f, "{}/{}/{}", self.owner, repo, self.id)?,
            None => write!(f, "{}/{}", self.owner, self.id)?,
        };
        Ok(())
    }
}
