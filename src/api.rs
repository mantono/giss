pub(crate) mod v4 {
    use lazy_static::lazy_static;
    use reqwest::blocking::Client;
    use std::time::Duration;

    const GITHUB_API_V4_URL: &str = "https://api.github.com/graphql";
    const USER_AGENT: &str = "giss";

    lazy_static! {
        pub static ref CLIENT: Client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap();
    }

    pub fn request<T: serde::de::DeserializeOwned>(token: &str, query: crate::search::GraphQLQuery) -> Result<T, u16> {
        log::debug!("{}", query.variables);

        let request: reqwest::blocking::Request = CLIENT
            .post(GITHUB_API_V4_URL)
            .header("User-Agent", USER_AGENT)
            .bearer_auth(token)
            .json(&query)
            .build()
            .expect("Failed to build query");

        let response: reqwest::blocking::Response = CLIENT.execute(request).unwrap();
        let status_code: u16 = response.status().as_u16();
        match status_code {
            200 => {
                log::debug!("GitHub API: {}", status_code);
                Ok(response.json().expect("Unable to parse body"))
            }
            _ => {
                let error: String = response.text().unwrap_or_default();
                log::error!("GitHub API: {} - {}", status_code, error);
                Err(status_code)
            }
        }
    }
}
