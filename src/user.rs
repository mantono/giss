use crate::GITHUB_API_V3_URL;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u64,
}

fn api_lookup_username(token: &str) -> User {
    let url: String = [GITHUB_API_V3_URL, "user"].join("/");
    let client = reqwest::Client::new();
    let mut response: reqwest::Response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .expect("Request to Github API failed when fetching user name");

    response.json::<User>().expect("Unable to parse GitHub user")
}

pub fn fetch_username(token: &str) -> String {
    match get_saved_username(token) {
        Some(username) => username,
        None => {
            let username: String = api_lookup_username(token).login;
            save_username(token, &username).expect("Unable to save username");
            username
        }
    }
}

fn save_username(token: &str, username: &str) -> Result<(), std::io::Error> {
    let token_hash: String = hash_token(token);
    let mut path: PathBuf = get_users_dir();
    std::fs::create_dir_all(&path).expect("Unable to create path");
    path.push(token_hash);
    let mut file: File = File::create(&path).expect("Unable to create file");
    file.write_all(username.as_bytes())
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.input(token);
    //  format!("{:02x}", hasher.result().as_slice().iter().format(""))
    format!("{:02x}", hasher.result())
}

fn get_users_dir() -> PathBuf {
    let mut path: PathBuf = dirs::home_dir().expect("Cannot find home dir");
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
