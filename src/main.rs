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
    let body = reqwest::get(&url).expect("Request failed");

    println!("{:?}", body);
}
