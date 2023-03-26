fn main() {
    // Fetch git tag/commit with `git describe --all --always --long --broken --dirty`
    let git_commit: String = git_commit();
    let cargo_data: String = cargo_data().join("\n");
    let output = format!(
        "Built with git commit {}and cargo configuration\n{}",
        git_commit, cargo_data
    );
    std::fs::write("target/build_data", output).expect("Unable to write buil_data file");
}

fn cmd(comand: &str) -> Option<String> {
    let parts: Vec<&str> = comand.split_whitespace().collect();
    let cmd: &str = parts.first().unwrap();
    let args: &[&str] = &parts[1..];

    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|res| String::from_utf8(res.stdout).ok())
        .ok()
        .flatten()
}

fn git_commit() -> String {
    cmd("git describe --all --always --long --broken --dirty")
        .unwrap_or_else(|| String::from("error"))
}

fn cargo_data() -> Vec<String> {
    let mut data: Vec<String> = std::env::vars()
        .into_iter()
        .filter(|(key, value)| include_env(key) && !value.is_empty())
        .map(|(key, value)| format!("{key}={value}"))
        .collect();

    data.sort();

    data
}

fn include_env(key: &str) -> bool {
    key.starts_with("CARGO") && !CARGO_EXCLUDE.contains(&key)
}

const CARGO_EXCLUDE: &[&str] = &["CARGO_MANIFEST_DIR", "CARGO_PKG_AUTHORS"];
