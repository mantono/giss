pub(crate) mod v4 {
    use lazy_static::lazy_static;
    use std::time::Duration;

    const GITHUB_API_V4_URL: &str = "https://api.github.com/graphql";
    const USER_AGENT: &str = "giss";

    lazy_static! {
        pub static ref CLIENT: reqwest::Client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap();
    }

    fn request<T: serde::de::DeserializeOwned>(token: &str, query: crate::search::GraphQLQuery) -> Result<T, u16> {
        log::debug!("{}", query.variables);

        let request: reqwest::Request = CLIENT
            .post(GITHUB_API_V4_URL)
            .header("User-Agent", USER_AGENT)
            .bearer_auth(token)
            .json(&query)
            .build()
            .expect("Failed to build query");

        let mut response: reqwest::Response = CLIENT.execute(request).unwrap();
        let status_code: u16 = response.status().as_u16();
        match status_code {
            200 => {
                log::debug!("GitHub API: {}", status_code);
                Ok(response.json().unwrap())
            }
            _ => {
                let error: String = response.text().unwrap_or_default();
                log::error!("GitHub API: {} - {}", status_code, error);
                Err(status_code)
            }
        }
    }
}
