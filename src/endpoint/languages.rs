use super::{Error, Result};
use crate::DeepLApi;
use serde::Deserialize;

/// Information about a supported language
#[derive(Deserialize)]
pub struct LangInfo {
    /// Language code
    pub language: String,
    /// Language name
    pub name: String,
    /// Denotes a target language supports formality
    pub supports_formality: Option<bool>,
}

/// Language type used to request supported languages
#[derive(Debug)]
pub enum LangType {
    /// Source language
    Source,
    /// Target language
    Target,
}

impl AsRef<str> for LangType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Source => "source",
            Self::Target => "target",
        }
    }
}

impl DeepLApi {
    ///
    /// Retrieve supported languages for a given [`LangType`]
    ///
    /// # Example
    ///
    /// ```rust
    /// let target_langs = deepl.languages(LangType::Target).await.unwrap();
    /// assert!(!target_langs.is_empty());
    ///
    /// let lang = target_langs.first().unwrap();
    /// println!("{}", lang.language); // BG
    /// println!("{}", lang.name); // Bulgarian
    /// ```
    pub async fn languages(&self, lang_type: LangType) -> Result<Vec<LangInfo>> {
        let q = vec![("type", lang_type.as_ref())];

        let resp = self
            .get(self.get_endpoint("languages"))
            .query(&q)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if !resp.status().is_success() {
            return super::extract_deepl_error(resp).await;
        }

        resp.json().await.map_err(|err| {
            Error::InvalidResponse(format!("convert json bytes to Rust type: {err}"))
        })
    }
}

#[tokio::test]
async fn test_get_languages() {
    let deepl = DeepLApi::with(&std::env::var("DEEPL_API_KEY").unwrap()).new();

    let langs = deepl.languages(LangType::Target).await.unwrap();
    assert!(!langs.is_empty());
}
