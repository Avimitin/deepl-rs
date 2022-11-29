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

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;
use typed_builder::TypedBuilder;

/// Representing error during interaction with DeepL
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid response: {0}")]
    InvalidReponse(String),

    #[error("request fail: {0}")]
    RequestFail(String),

    #[error("fail to read file {0}: {1}")]
    ReadFileError(String, tokio::io::Error),
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

/// Configure how to upload the document to DeepL API.
///
/// # Example
///
/// ```rust
/// let prop = UploadDocumentProp::builder()
///     .source_lang(Lang::EN_GB)
///     .target_lang(Lang::ZH)
///     .file_path("/path/to/document.pdf")
///     .filename("Foo Bar Baz")
///     .formality(Formality::Default)
///     .glossary_id("def3a26b-3e84-45b3-84ae-0c0aaf3525f7")
///     .build();
/// ...
/// ```
#[derive(TypedBuilder, Serialize)]
#[builder(doc)]
pub struct UploadDocumentProp {
    /// Language of the text to be translated, optional
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<Lang>,
    /// Language into which text should be translated, required
    target_lang: Lang,
    #[builder(default, setter(skip))]
    file: Vec<u8>,
    /// Path of the file to be translated, required
    #[serde(skip)]
    #[builder(setter(transform = |p: &str| PathBuf::from(p)))]
    file_path: PathBuf,
    /// Name of the file, optional
    #[builder(default, setter(transform = |f: &str| Some(f.to_string())))]
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
    /// Sets whether the translated text should lean towards formal or informal language.
    /// This feature currently only works for target languages DE (German), FR (French),
    /// IT (Italian), ES (Spanish), NL (Dutch), PL (Polish), PT-PT, PT-BR (Portuguese)
    /// and RU (Russian). Setting this parameter with a target language that does not
    /// support formality will fail, unless one of the prefer_... options are used. optional
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    formality: Option<Formality>,
    /// A unique ID assigned to your accounts glossary. optional
    #[builder(default, setter(transform = |g: &str| Some(g.to_string())))]
    #[serde(skip_serializing_if = "Option::is_none")]
    glossary_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Formality {
    Default,
    More,
    Less,
    PreferMore,
    PreferLess,
}

impl AsRef<str> for Formality {
    fn as_ref(&self) -> &str {
        match self {
            Self::Default => "default",
            Self::More => "more",
            Self::Less => "less",
            Self::PreferMore => "prefer_more",
            Self::PreferLess => "prefer_less",
        }
    }
}

/// Response from api/v2/document
#[derive(Serialize, Deserialize)]
pub struct DocumentUploadResp {
    /// A unique ID assigned to the uploaded document and the translation process.
    /// Must be used when referring to this particular document in subsequent API requests.
    document_id: String,
    /// A unique key that is used to encrypt the uploaded document as well as the resulting
    /// translation on the server side. Must be provided with every subsequent API request
    /// regarding this particular document.
    document_key: String,
}

/// Response from api/v2/document/$ID
#[derive(Deserialize)]
pub struct DocumentStatusResp {
    document_id: String,
    status: DocumentTranslateStatus,
    seconds_remaining: Option<u64>,
    billed_characters: u64,
    error_message: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentTranslateStatus {
    Queued,
    Translating,
    Done,
    Error,
}

/// A struct that contains necessary data
#[derive(Debug)]
pub struct DeepLApi {
    client: reqwest::Client,
    key: String,
    endpoint: reqwest::Url,
}

impl DeepLApi {
    /// Create a new api instance with auth key. If you are paid user, pass `true` into the second
    /// parameter.
    pub fn new(key: &str, is_pro: bool) -> Self {
        let endpoint = if is_pro {
            "https://api.deepl.com/v2/"
        } else {
            "https://api-free.deepl.com/v2/"
        };

        let endpoint = reqwest::Url::parse(endpoint).unwrap();
        Self {
            endpoint,
            client: reqwest::Client::new(),
            key: format!("DeepL-Auth-Key {}", key),
        }
    }

    fn post(&self, url: reqwest::Url) -> reqwest::RequestBuilder {
        self.client.post(url).header("Authorization", &self.key)
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
            .post(self.endpoint.join("translate").unwrap())
            .form(&param)
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
            .post(self.endpoint.join("usage").unwrap())
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

    /// Upload document to DeepL server, returning a document ID and key which can be used
    /// to query the translation status and to download the translated document once
    /// translation is complete.
    pub async fn upload_document(
        &self,
        mut prop: UploadDocumentProp,
    ) -> Result<DocumentUploadResp> {
        let file = tokio::fs::read(&prop.file_path).await.map_err(|err| {
            Error::ReadFileError(prop.file_path.to_str().unwrap().to_string(), err)
        })?;
        prop.file = file;
        let res = self
            .post(self.endpoint.join("document").unwrap())
            .form(&prop)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?
            .bytes()
            .await
            .map_err(|err| Error::InvalidReponse(format!("fail to decode response body: {err}")))?;

        let upload_resp: DocumentUploadResp = serde_json::from_slice(&res)
            .map_err(|err| Error::InvalidReponse(format!("response is not a valid: {err}")))?;

        Ok(upload_resp)
    }

    pub async fn check_document_status(
        &self,
        ident: &DocumentUploadResp,
    ) -> Result<DocumentTranslateStatus> {
        let form = [("document_key", ident.document_key.as_str())];
        let url = self
            .endpoint
            .join(&format!("document/{}", ident.document_id))
            .unwrap();
        let res = self
            .post(url)
            .form(&form)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?
            .bytes()
            .await
            .map_err(|err| Error::InvalidReponse(format!("response is not valid: {err}")))?;

        let status: DocumentTranslateStatus = serde_json::from_slice(&res)
            .map_err(|err| Error::InvalidReponse(format!("response is not JSON: {err}")))?;

        Ok(status)
    }
}

#[tokio::test]
async fn test_translator() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key, false);
    let response = api.translate("Hello World", None, Lang::ZH).await.unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(translated_results[0].text, "你好，世界");
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}

#[tokio::test]
async fn test_usage() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key, false);
    let response = api.get_usage().await.unwrap();

    assert_ne!(response.character_count, 0);
}

#[tokio::test]
async fn test_upload_document() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key, false);
    let upload_option = UploadDocumentProp::builder()
        .target_lang(Lang::EN_US)
        .file_path("./test.txt")
        .build();
    let response = api.upload_document(upload_option).await.unwrap();
    let status = api.check_document_status(&response).await.unwrap();
}
