mod cmd;
mod issue;

use cmd::cmd::Command as carg;
use cmd::cmd::{cmd_has, cmd_read};
use issue::issue::{Assignee, Issue, IssueRequest, Label};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    match carg::parse() {
        Some(carg::Add) => {
            let token: String = read_token();
            let repo: String = read_repo();
            let read_issue: IssueRequest = read_issue(&repo);
            create_issue(&repo, &token, &read_issue)
        }
        Some(carg::List) => {
            let token: String = read_token();
            let repo: String = read_repo();
            list_issues(&repo, &token);
        }
        Some(carg::Help) => cmd::cmd::print_help_text(),
        Some(carg::Invalid(invalid_arg)) => cmd::cmd::print_invalid_arg(&invalid_arg),
        None => cmd::cmd::print_no_arg(),
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

    let file_content: String = fs::read_to_string(path).expect("Could not read issue file");
    let lines: Vec<&str> = file_content
        .lines()
        .filter(|line| !line.starts_with("#") && !line.trim().is_empty())
        .collect();

    let title: &str = lines.first().expect("Issue was empty");
    let body: Vec<String> = lines
        .iter()
        .skip(1)
        .filter(|line| !line.starts_with("labels:") && !line.starts_with("assignees:"))
        .map(|&x| x.to_string())
        .collect::<Vec<String>>();

    let body: Option<String> = match body.is_empty() {
        true => None,
        false => Some(body.join("\n")),
    };

    let labels: Vec<String> = read_attribute("labels:", &lines);
    let assignees: Vec<String> = read_attribute("assignees:", &lines);

    IssueRequest {
        title: title.to_string(),
        body: body,
        labels: labels,
        assignees: assignees,
    }
}

fn read_attribute(keyword: &str, lines: &Vec<&str>) -> Vec<String> {
    let attribute_line: String = lines
        .iter()
        .filter(|line| line.starts_with(keyword))
        .map(|&line| line.to_string())
        .take(1)
        .collect::<Vec<String>>()
        .first()
        .unwrap_or(&String::from(""))
        .clone();

    attribute_line
        .trim_start_matches(keyword)
        .split_terminator(",")
        .map(|value| value.trim().to_string())
        .collect()
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
    println!("Got issue {:?}", issue);
    let url: String = [GITHUB_API, "repos", repo, "issues"].join("/");
    let client = reqwest::Client::new();
    let mut response: reqwest::Response = client
        .post(&url)
        .bearer_auth(token)
        .json(&issue)
        .send()
        .expect("Failed to submit issue");

    if !response.status().is_success() {
        let body_response: String = response.text().unwrap_or(String::from(""));
        println!("Error {}: {}", response.status(), body_response);
    }

    let exit_code: i32 = match response.status().as_u16() {
        200..=299 => 0,
        400..=499 => 1,
        500..=599 => 2,
        _ => 9,
    };

    std::process::exit(exit_code);
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
