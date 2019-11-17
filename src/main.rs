#[macro_use]
extern crate clap;
extern crate regex;

mod args;
mod issue;

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
    let args: clap::ArgMatches = args::args::parse_args();
    let action: &str = args.value_of("action").expect("Action must be present");
    let token: String = args
        .value_of("token")
        .expect("No token was present")
        .to_string();

    let targets: Vec<&str> = args
        .values_of("target")
        .expect("Target must be present")
        .collect();

    let target: String = targets
        .first()
        .expect("At least one target must be present")
        .to_string();

    match action {
        "create" => {
            let read_issue: IssueRequest = read_issue(&target);
            create_issue(&target, &token, &read_issue)
        }
        "list" => list_issues(&target, &token),
        _ => panic!("This should never happen"),
    }
}

const FILE_CONTENT: &str = "
# Insert title and body above for issue. First line will automatically be interpreted
# as the title of the subject and following lines will be the body of the issue.
# Optionally, labels can be added with `labels: duplicate, jar, my-favourite-label` on a separate
# line and assginees with `assignees: @assignedperson, @some-other-poor-fellow`. Lines
# that starts with a '#' will be ignored.
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
