use crate::{search::GraphQLQuery, AppErr};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use std::{fs::File, str::FromStr};

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Username(pub String);

impl Username {
    pub async fn from_token(token: &str) -> Result<Username, AppErr> {
        match get_saved_username(token) {
            Some(username) => Ok(Username(username)),
            None => {
                let username: String = api_lookup_username(token).await?.login;
                save_username(token, &username)?;
                Ok(Username(username))
            }
        }
    }
}

impl FromStr for Username {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Username(s.to_string()))
    }
}

impl From<std::io::Error> for AppErr {
    fn from(_: std::io::Error) -> Self {
        AppErr::TokenWriteError
    }
}

impl From<reqwest::Error> for AppErr {
    fn from(e: reqwest::Error) -> Self {
        log::error!("Request failed {}", e);
        AppErr::ApiError
    }
}

#[derive(Debug, Deserialize)]
struct Root {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    pub viewer: Viewer,
}

#[derive(Debug, Deserialize)]
struct Viewer {
    login: String,
    id: String,
}

async fn api_lookup_username(token: &str) -> Result<User, AppErr> {
    let query = GraphQLQuery {
        variables: serde_json::Value::Null,
        query: String::from(include_str!("../data/graphql/queries/get_user.graphql")),
        operation_name: String::from("GetUser"),
    };

    let root: Root = crate::api::v4::request(token, query).await?;
    let user = User {
        login: root.data.viewer.login,
        id: root.data.viewer.id,
    };
    Ok(user)
}

fn save_username(token: &str, username: &str) -> Result<(), std::io::Error> {
    let token_hash: String = hash_token(token);
    let mut path: PathBuf = get_users_dir();
    std::fs::create_dir_all(&path)?;
    path.push(token_hash);
    let mut file: File = File::create(&path)?;
    file.write_all(username.as_bytes())
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input(token);
    format!("{:02x}", hasher.result())
}

fn get_users_dir() -> PathBuf {
    let mut path: PathBuf = dirs_next::home_dir().expect("Cannot find home dir");
    path.push([".config", "giss", "usernames"].join("/"));
    path
}

fn get_saved_username(token: &str) -> Option<String> {
    let token_hash: String = hash_token(token);
    let mut path: PathBuf = get_users_dir();
    path.push(token_hash);
    if path.exists() {
        let content = std::fs::read_to_string(path).expect("Unable to read content of file");
        Some(content)
    } else {
        None
    }
}
