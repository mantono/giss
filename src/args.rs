use std::fs;
use std::path::{Path, PathBuf};

pub fn read_repo_from_file() -> Option<String> {
    let current_path: &Path = Path::new(".");
    let repo_root: PathBuf = match giro::git_root(&current_path).unwrap() {
        Some(root) => root,
        None => return None,
    };
    let config_file: PathBuf = repo_root.join(".git").join("config");
    log::debug!("Using Git config file: '{:?}'", config_file);
    let file_content: String =
        fs::read_to_string(config_file).expect("Could not find a git config");

    let lines: Vec<&str> = file_content
        .lines()
        .filter(|f| f.contains("github.com"))
        .collect();

    let repo: &str = lines
        .first()
        .expect("No Github repository found")
        .split_terminator(':')
        .last()
        .expect("No match");

    Some(repo.trim_end_matches(".git").to_string())
}
