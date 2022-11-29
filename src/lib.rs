//! # deepl-rs
//!
//! Deepl-rs is a simple wrapper for providing simple function to make request to the DeepL API endpoint
//! and typed response. This is still a incomplete library, please open a issue on GitHub to tell
//! me what feature you want.
//!
//! See the README for usage.
//!
//! # License
//!
//! This project is licensed under MIT license.
mod lang;

pub use lang::Lang;

use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

const TRANSLATE_TEXT_ENDPOINT: &str = "https://api-free.deepl.com/v2/translate";
const USAGE_ENDPOINT: &str = "https://api-free.deepl.com/v2/usage";

/// Representing error during interaction with DeepL
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid language code {0}")]
    InvalidLang(String),

    #[error("invalid response: {0}")]
    InvalidReponse(String),

    #[error("request fail: {0}")]
    RequestFail(String),
}

type Result<T, E = Error> = core::result::Result<T, E>;

/// Response from basic translation API
#[derive(Deserialize)]
pub struct DeepLApiResponse {
    pub translations: Vec<Sentence>,
}

/// Translated result for a sentence
#[derive(Deserialize)]
pub struct Sentence {
    pub detected_source_language: Lang,
    pub text: String,
}

/// Reponse from the usage API
#[derive(Deserialize)]
pub struct UsageReponse {
    pub character_count: u64,
    pub character_limit: u64,
}

/// A struct that contains necessary data
#[derive(Debug)]
pub struct DeepLApi {
    client: reqwest::Client,
    key: String,
}

impl DeepLApi {
    /// Create a new api instance with auth key
    pub fn new(key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            key: format!("DeepL-Auth-Key {}", key),
        }
    }

    /// Translate the given text into expected target language. Source language is optional
    /// and can be detemined by DeepL API.
    ///
    /// # Error
    ///
    /// Return error if the http request fail
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepl::{DeepLApi, Lang};
    ///
    /// let api = DeepLApi::new("YOUR AUTH KEY");
    /// api.translate("Hello World", None, Lang::ZH).await.unwrap();
    /// ```
    pub async fn translate(
        &self,
        text: &str,
        translate_from: Option<Lang>,
        translate_into: Lang,
    ) -> Result<DeepLApiResponse> {
        let mut param = HashMap::new();
        param.insert("text", text);
        if let Some(ref la) = translate_from {
            param.insert("source_lang", la.as_ref());
        }
        param.insert("target_lang", translate_into.as_ref());

        let response = self
            .client
            .post(TRANSLATE_TEXT_ENDPOINT)
            .form(&param)
            .header("Authorization", &self.key)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        let response = response
            .bytes()
            .await
            .map_err(|err| Error::InvalidReponse(format!("decoding http body to byte: {err}")))?;
        let response: DeepLApiResponse = serde_json::from_slice(&response).map_err(|err| {
            Error::InvalidReponse(format!("convert json bytes to Rust type: {err}"))
        })?;

        Ok(response)
    }

    /// Get the current DeepL API usage
    pub async fn get_usage(&self) -> Result<UsageReponse> {
        let response = self
            .client
            .post(USAGE_ENDPOINT)
            .header("Authorization", &self.key)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        let response = response
            .bytes()
            .await
            .map_err(|err| Error::InvalidReponse(format!("decoding http body to byte: {err}")))?;

        let usage: UsageReponse = serde_json::from_slice(&response).map_err(|err| {
            Error::InvalidReponse(format!("convert json bytes to Rust type: {err}"))
        })?;

        Ok(usage)
    }
}

#[tokio::test]
async fn test_translator() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key);
    let response = api.translate("Hello World", None, Lang::ZH).await.unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(translated_results[0].text, "你好，世界");
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}

#[tokio::test]
async fn test_usage() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key);
    let response = api.get_usage().await.unwrap();

    assert_ne!(response.character_count, 0);
}
