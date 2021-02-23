use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt, str::FromStr};

#[derive(Debug)]
pub enum Target {
    Organization(String),
    Repository(String, String),
}

lazy_static! {
    static ref TARGET: Regex = Regex::new(r"[\w\-\.]+").unwrap();
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<String> = TARGET
            .find_iter(s)
            .into_iter()
            .map(|x| x.as_str().to_string())
            .collect();
        match parts.len() {
            1 => Ok(Target::Organization(parts[0].clone())),
            2 => Ok(Target::Repository(parts[0].clone(), parts[1].clone())),
            _ => Err(format!("Could not resolve a valid target from '{}'", s)),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Organization(name) => write!(f, "org:{}", name),
            Target::Repository(owner, name) => write!(f, "repo:{}/{}", owner, name),
        }
    }
}

impl Clone for Target {
    fn clone(&self) -> Target {
        match self {
            Target::Organization(name) => Target::Organization(name.clone()),
            Target::Repository(owner, name) => Target::Repository(owner.clone(), name.clone()),
        }
    }
}
