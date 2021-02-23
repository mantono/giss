pub(crate) mod v4 {
    use lazy_static::lazy_static;
    use reqwest::Client;
    use std::time::Duration;

    use super::ApiError;

    const GITHUB_API_V4_URL: &str = "https://api.github.com/graphql";
    const USER_AGENT: &str = "giss";

    lazy_static! {
        pub static ref CLIENT: Client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap();
    }

    pub async fn request<T: serde::de::DeserializeOwned>(
        token: &str,
        query: crate::search::GraphQLQuery,
    ) -> Result<T, ApiError> {
        log::debug!("{}", query.variables);

        let request: reqwest::Request = CLIENT
            .post(GITHUB_API_V4_URL)
            .header("User-Agent", USER_AGENT)
            .bearer_auth(token)
            .json(&query)
            .build()
            .expect("Failed to build query");

        let response: reqwest::Response = CLIENT.execute(request).await?;
        let status_code: u16 = response.status().as_u16();
        match status_code {
            200 => {
                log::debug!("GitHub API: {}", status_code);
                Ok(response.json().await?)
            }
            _ => {
                let error: String = response.text().await?;
                log::error!("GitHub API: {} - {}", status_code, error);
                Err(ApiError::Response(status_code))
            }
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        match e.status() {
            Some(code) => ApiError::Response(code.as_u16()),
            None => ApiError::NoResponse(e.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    NoResponse(String),
    Response(u16),
}
