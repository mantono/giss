use std::env;
use std::fs;

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    let file_content: String = fs::read_to_string(".git/config").expect("Could not find a git config");
    let lines: Vec<&str> = file_content.lines()
        .filter(|f| f.contains("github.com"))
        .collect();

    lines.iter().for_each(|f| println!("{:?}", f));

    let repo: &str = lines.first()
        .expect("No Github repoistory found")
        .split_terminator(":")
        .last()
        .expect("No match");

    println!("{}", repo);
    // let url = [GITHUB_API, "repos", owner, repo, "issues"].join("/");
    // let body = reqwest::get(url)
    //     .await()?
    //     .text()
    //     .await()?;
}