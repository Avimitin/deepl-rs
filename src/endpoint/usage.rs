use super::Result;
use crate::{DeepLApi, Error};
use serde::Deserialize;

/// Response from the usage API
#[derive(Deserialize)]
pub struct UsageResponse {
    pub character_count: u64,
    pub character_limit: u64,
}

impl DeepLApi {
    /// Get the current DeepL API usage
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepl::DeepLApi
    ///
    /// let api = DeepLApi::new("Your DeepL Token", false);
    /// let response = api.get_usage().await.unwrap();
    ///
    /// assert_ne!(response.character_count, 0);
    /// ```
    pub async fn get_usage(&self) -> Result<UsageResponse> {
        let response = self
            .post(self.get_endpoint("usage"))
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if !response.status().is_success() {
            return super::extract_deepl_error(response).await;
        }

        let response: UsageResponse = response.json().await.map_err(|err| {
            Error::InvalidResponse(format!("convert json bytes to Rust type: {err}"))
        })?;

        Ok(response)
    }
}

#[tokio::test]
async fn test_usage() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::with(&key).new();
    let response = api.get_usage().await.unwrap();

    assert_ne!(response.character_count, 0);
}
