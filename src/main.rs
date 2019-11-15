use serde::Deserialize;
use serde::Serialize;

use std::env;
use std::env::temp_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
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
    let token: Option<String> = cmd_read("--token");
    match token {
        Some(t) => t.clone(),
        None => env!("GITHUB_TOKEN").to_string(),
    }
}

fn read_repo() -> String {
    let repo_arg: Option<String> = cmd_read("--repo");
    match repo_arg {
        Some(repo) => repo,
        None => read_repo_from_file(),
    }
}

fn read_repo_from_file() -> String {
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

fn cmd_read(key: &str) -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let value: Option<&String> = args.iter().skip_while(|i| !i.contains(key)).skip(1).next();
    value.cloned()
}

const FILE_CONTENT: &str = "
# Insert title and body above for issue. First line will automatically be interpreted
# as the title of the subject and following lines will be the body of the issue.
# Optionally, labels can be added with `labels: duplicate, jar, my-favourite-label` on a separate
# line and assginees with `assignees: @assignedperson, @some-other-poor-fellow`.
";

fn read_issue(repo: &String) -> IssueRequest {
    let mut path: PathBuf = env::temp_dir();
    path.push(repo);
    let timestamp: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("If you see this message, your system clock is wrong or you are in a Back to The Future movie")
        .as_millis();

    fs::create_dir_all(&path).expect("Unable to create directory");
    path.push(timestamp.to_string());
    println!("{:?}", path);
    let mut file: File = File::create(&path).expect("Could not create file");
    println!("{:?}", path);
    let result = file.write_all(FILE_CONTENT.as_bytes());

    let cmd: String = format!("$(env $EDITOR {:?})", path.to_str().expect("Is not empty"));
    let execution_result: std::process::ExitStatus = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .expect("failed to execute process")
        .wait()
        .expect("Failed to get exit status");

    if !execution_result.success() {
        panic!("Editing commit message failed")
    }

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

    let issues: Vec<Issue> = response.json().expect("Unable to process body in response");

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
    let assignees: String = issue
        .assignees
        .iter()
        .map(|a: &Assignee| &a.login)
        .map(|s: &String| format!("{}{}", "@", s))
        .collect::<Vec<String>>()
        .join(", ");

    let labels: String = issue
        .labels
        .iter()
        .map(|l: &Label| &l.name)
        .map(|s: &String| format!("{}{}", "#", s))
        .collect::<Vec<String>>()
        .join(", ");

    let extra: String = vec![title, assignees, labels]
        .iter()
        .filter(|i| !i.is_empty())
        .map(|s| s.clone())
        .collect::<Vec<String>>()
        .join(" | ");

    println!("#{} {}", issue.number, extra);
}

fn truncate(string: String, max_length: usize) -> String {
    let new_length: usize = std::cmp::min(string.len(), max_length);
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
    assignees: Vec<Assignee>,
    labels: Vec<Label>,
}

#[derive(Debug, Deserialize)]
struct Label {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Assignee {
    login: String,
}

#[derive(Debug, Serialize)]
struct IssueRequest {
    title: String,
    body: String,
    labels: Vec<String>,
    assignees: Vec<String>,
}
