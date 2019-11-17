use serde::Deserialize;
use serde::Serialize;

fn get_user(token: &String) -> User {
    let url: String = [GITHUB_API, "user"].join("/");
    let client = reqwest::Client::new();
    let mut response: reqwest::Response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .expect("Request to Github API failed");

    let user: User = response.json().expect("Unable to process body in response");
    user
}

#[derive(Debug, Deserialize)]
pub struct User {
    login: String,
    id: u64,
}
