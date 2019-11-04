use git2::Repository;

const GITHUB_API: &str = "https://api.github.com";

fn main() {
    let repo: Repository = Repository::open_from_env().expect("Found no repo");
    let owner: &str = repo.remotes()
        .expect("Found no remotes")
        .iter()        
        .filter_map(|f| f)
        .find(|r| r.contains("github.com"))
        .expect("Found no remote for GitHub");
    // let url = [GITHUB_API, "repos", owner, repo, "issues"].join("/");
    // let body = reqwest::get(url)
    //     .await()?
    //     .text()
    //     .await()?;

    println!("body = {:?}", &owner);
}
