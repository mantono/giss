use serde::Deserialize;
use serde::Serialize;

use std::env;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    let token: String = read_token();
    let repo: String = read_repo();

    if cmd_has("add") {
        let read_isse: IssueRequest = read_issue(&repo);
        let test_issue = IssueRequest {
            title: "A title".to_string(),
            body: "A body".to_string(),
            labels: vec!["test".to_string()],
            assignees: vec![],
        };
        create_issue(&repo, &token, &test_issue)
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

fn read_issue(repo: &String) -> IssueRequest {
    let mut path: PathBuf = env::temp_dir();
    path.push(repo);
    let timestamp: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("If you see this message, your system clock is wrong or you are in a Back to The Future movie")
        .as_millis();

    path.push(timestamp.to_string());

    Command::new("sh")
        .arg("-c")
        .arg("$(env $EDITOR)")
        .arg(path)
        .output()
        .expect("failed to execute process");

    IssueRequest {
        title: "A title".to_string(),
        body: "A body".to_string(),
        labels: vec!["test".to_string()],
        assignees: vec![],
    }
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

fn create_issue(repo: &String, token: &String, issue: &IssueRequest) {
    let url: String = [GITHUB_API, "repos", repo, "issues"].join("/");
    let client = reqwest::Client::new();
    let mut response: reqwest::Response = client
        .post(&url)
        .bearer_auth(token)
        .json(&issue)
        .send()
        .expect("Failed to submit issue");
}

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

#[derive(Debug, Serialize)]
struct IssueRequest {
    title: String,
    body: String,
    labels: Vec<String>,
    assignees: Vec<String>,
}
