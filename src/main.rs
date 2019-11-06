use serde::Deserialize;

use std::env;
use std::fs;

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    let file_content: String =
        fs::read_to_string(".git/config").expect("Could not find a git config");

    let args: Vec<String> = env::args().collect();
    let token: Option<&String> = args
        .iter()
        .skip_while(|i| !i.contains("--token"))
        .skip(1)
        .next();

    let token: String = match token {
        Some(t) => t.clone(),
        None => env!("GITHUB_TOKEN").to_string(),
    };

    println!("{}", token);

    //.skip_while(|i| !i.contains("--token"));

    let lines: Vec<&str> = file_content
        .lines()
        .filter(|f| f.contains("github.com"))
        .collect();

    lines.iter().for_each(|f| println!("{:?}", f));

    let repo: &str = lines
        .first()
        .expect("No Github repoistory found")
        .split_terminator(":")
        .last()
        .expect("No match");

    let repo: &str = repo.trim_end_matches(".git");

    println!("{}", repo);

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
