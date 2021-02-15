use crate::{api::v4::CLIENT, AppErr};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use std::{fs::File, str::FromStr};

const GITHUB_API_V3_URL: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Clone)]
pub struct Username(pub String);

impl Username {
    pub fn from_token(token: &str) -> Result<Username, AppErr> {
        match get_saved_username(token) {
            Some(username) => Ok(Username(username)),
            None => {
                let username: String = api_lookup_username(token)?.login;
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

fn api_lookup_username(token: &str) -> Result<User, AppErr> {
    let url: String = [GITHUB_API_V3_URL, "user"].join("/");
    let response: reqwest::blocking::Response = CLIENT.get(&url).bearer_auth(token).send()?;
    Ok(response.json::<User>()?)
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
