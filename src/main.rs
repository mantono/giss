use serde::Deserialize;

use std::env;
use std::fs;

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    let token: String = read_token();
    let repo: String = read_repo();

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

    let issues: Vec<Issue> = response.json().expect("No body found in response");

    issues
        .iter()
        .filter(|i| i.state == "open")
        .for_each(print_issue);
}

fn create_issue(repo: &String, token: &String) {}

fn print_issue(issue: &Issue) {
    let title: String = truncate(issue.title.clone(), 50);
    let body: String = truncate(issue.body.clone(), 200);
    println!("#{} {} || {}", issue.number, title, body);
}

fn truncate(string: String, length: usize) -> String {
    let new_length: usize = std::cmp::min(string.len(), length);
    if new_length < string.len() {
        string[..new_length].to_string()
    } else {
        string
    }
}

#[derive(Debug, Deserialize)]
struct Issue {
    url: String,
    id: u64,
    number: u32,
    title: String,
    body: String,
    updated_at: String,
    state: String,
    comments: u32,
}
