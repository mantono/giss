use serde::Deserialize;

use std::env;
use std::fs;

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    let token: String = read_token();
    println!("{}", token);

    let repo: String = read_repo();
    println!("{}", repo);

    println!("has add: {}", cmd_has("add"));
    if cmd_has("add") {
        create_issue(&repo, &token)
    } else {
        list_issues(&repo, &token)
    }
}

fn read_token() -> String {
    let args: Vec<String> = env::args().collect();
    let token: Option<&String> = args
        .iter()
        .skip_while(|i| !i.contains("--token"))
        .skip(1)
        .next();

    match token {
        Some(t) => t.clone(),
        None => env!("GITHUB_TOKEN").to_string(),
    }
}

fn read_repo() -> String {
    let file_content: String =
        fs::read_to_string(".git/config").expect("Could not find a git config");

    let lines: Vec<&str> = file_content
        .lines()
        .filter(|f| f.contains("github.com"))
        .collect();

    let repo: &str = lines
        .first()
        .expect("No Github repoistory found")
        .split_terminator(":")
        .last()
        .expect("No match");

    repo.trim_end_matches(".git").to_string()
}

fn cmd_has(key: &str) -> bool {
    env::args().any(|i| i == key)
}

fn list_issues(repo: &String, token: &String) {
    let url: String = [GITHUB_API, "repos", repo, "issues"].join("/");
    let client = reqwest::Client::new();
    let mut response: reqwest::Response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .expect("Request to Github API failed");

    let body: Vec<Issue> = response.json().expect("No body found in response");

    println!("{:?}", body);
}

fn create_issue(repo: &String, token: &String) {}

#[derive(Debug, Deserialize)]
struct Issue {
    url: String,
    id: u64,
    title: String,
    body: String,
    updated_at: String,
    state: String,
    comments: u32,
}
