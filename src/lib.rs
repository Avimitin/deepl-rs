//! # deepl-rs
//!
//! Deepl-rs is a simple wrapper for providing simple function to make request to the DeepL API endpoint
//! and typed response.
//!
//! This is still a **incomplete** library, please open a issue on GitHub to tell
//! me what feature you want.
//!
//! # Usage
//!
//! ```rust
//! let api = DeepLApi::new("Your DeepL Token", false);
//! let prop = TranslateTextProp::builder().target_lang(Lang::ZH).build();
//! let response = api.translate("Hello World", &prop).await.unwrap();
//!
//! assert!(!response.translations.is_empty());
//!
//! let translated_results = response.translations;
//! assert_eq!(translated_results[0].text, "你好，世界");
//! assert_eq!(translated_results[0].detected_source_language, Lang::EN);
//! ```
//!
//! See [`DeepLApi`] for detail usage.
//!
//! # License
//!
//! This project is licensed under MIT license.
//!

mod endpoint;
mod lang;

pub use endpoint::Error;
pub use lang::Lang;
pub use reqwest;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use typed_builder::TypedBuilder;

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

/// Response from the usage API
#[derive(Deserialize)]
pub struct UsageResponse {
    pub character_count: u64,
    pub character_limit: u64,
}

/// Response from api/v2/document/$ID
#[derive(Deserialize, Debug)]
pub struct DocumentStatusResp {
    /// A unique ID assigned to the uploaded document and the requested translation process.
    /// The same ID that was used when requesting the translation status.
    pub document_id: String,
    /// A short description of the state the document translation process is currently in.
    /// See [`DocumentTranslateStatus`] for more.
    pub status: DocumentTranslateStatus,
    /// Estimated number of seconds until the translation is done.
    /// This parameter is only included while status is "translating".
    pub seconds_remaining: Option<u64>,
    /// The number of characters billed to your account.
    pub billed_characters: Option<u64>,
    /// A short description of the error, if available. Note that the content is subject to change.
    /// This parameter may be included if an error occurred during translation.
    pub error_message: Option<String>,
}

/// Possible value of the document translate status
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentTranslateStatus {
    /// The translation job is waiting in line to be processed
    Queued,
    /// The translation is currently ongoing
    Translating,
    /// The translation is done and the translated document is ready for download
    Done,
    /// An irrecoverable error occurred while translating the document
    Error,
}

impl DocumentTranslateStatus {
    pub fn is_done(&self) -> bool {
        self == &Self::Done
    }
}

/// A struct that contains necessary data. If you don't have any other requirement, you can called
/// the `DeepLApi::new` function to create an instance. If you want to customize this wrapper, you
/// can use the `DeepLApi::builder` function to set the fields.
///
/// # Example
///
/// ```
/// // simple API creation
/// let deepl = DeepLApi::new();
///
/// // **OR** customize it
/// let duration = std::time::Duration::from_secs(30);
/// let client = reqwest::Client::builder().timeout(duration).build().unwrap();
/// let deepl = DeepLApi::builder()
///                 .key("Your DeepL Key")       // set the auth key
///                 .endpoint(true)              // use the pro api
///                 .client(client)              // use a http client with 30 secs timeout
///                 .build();
/// ```
#[derive(Debug, TypedBuilder)]
#[builder(builder_type_doc = "Builder for a completely customized API wrapper")]
pub struct DeepLApi {
    #[builder(default, setter(doc = "Set a customized reqwest client"))]
    client: reqwest::Client,
    #[builder(setter(
        doc = "Set the API auth token",
        transform = |s: impl ToString| s.to_string(),
    ))]
    key: String,
    #[builder(
        default = reqwest::Url::parse("https://api-free.deepl.com/v2/").unwrap(),
        setter(
            doc = "Set this field to true if you are paid user, default using api-free API",
            transform = |pro: bool| {
                let url = if pro {
                    "https://api.deepl.com/v2/"
                } else {
                    "https://api-free.deepl.com/v2/"
                };
                reqwest::Url::parse(url).unwrap()
            }
        )
    )]
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
            .post(self.endpoint.join("usage").unwrap())
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if !response.status().is_success() {
            return Self::extract_deepl_error(response).await;
        }

        let response: UsageResponse = response.json().await.map_err(|err| {
            Error::InvalidResponse(format!("convert json bytes to Rust type: {err}"))
        })?;

        Ok(response)
    }

    /// Upload document to DeepL server, returning a document ID and key which can be used
    /// to query the translation status and to download the translated document once
    /// translation is complete.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepl::DeepLApi
    ///
    /// let api = DeepLApi::new(&key, false);
    ///
    /// // configure upload option
    /// let upload_option = UploadDocumentProp::builder()
    ///     .source_lang(Lang::EN_GB)
    ///     .target_lang(Lang::ZH)
    ///     .file_path("./hamlet.txt")
    ///     .filename("Hamlet.txt")
    ///     .formality(Formality::Default)
    ///     .glossary_id("def3a26b-3e84-45b3-84ae-0c0aaf3525f7")
    ///     .build();
    ///
    /// // Upload the file to DeepL
    /// let response = api.upload_document(upload_option).await.unwrap();
    ///
    /// // Query the translate status
    /// let mut status = api.check_document_status(&response).await.unwrap();
    ///
    /// // wait for translation
    /// loop {
    ///     if status.status.is_done() {
    ///         break;
    ///     }
    ///     if let Some(msg) = status.error_message {
    ///         eprintln!("{}", msg);
    ///         break;
    ///     }
    ///     tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    ///     status = api.check_document_status(&response).await.unwrap();
    /// }
    ///
    /// // After translation done, download it to "translated.txt"
    /// let path = api
    ///     .download_document(&response, "translated.txt", None)
    ///     .await
    ///     .unwrap();
    ///
    /// // See whats in it
    /// let content = tokio::fs::read_to_string(path).await.unwrap();
    /// // ...
    /// ```
    pub async fn upload_document(
        &self,
        fp: impl Into<std::path::PathBuf>,
        target_lang: Lang,
    ) -> endpoint::document::UploadDocumentRequester {
        endpoint::document::UploadDocumentRequester::new(self, fp.into(), target_lang)
    }

    /// Check the status of document, returning [`DocumentStatusResp`] if success.
    pub async fn check_document_status(
        &self,
        ident: &DocumentUploadResp,
    ) -> Result<DocumentStatusResp> {
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
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if !res.status().is_success() {
            return Self::extract_deepl_error(res).await;
        }

        let status: DocumentStatusResp = res
            .json()
            .await
            .map_err(|err| Error::InvalidResponse(format!("response is not JSON: {err}")))?;

        Ok(status)
    }

    async fn open_file_to_write(p: &Path) -> Result<tokio::fs::File> {
        let open_result = tokio::fs::OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(p)
            .await;

        if let Ok(file) = open_result {
            return Ok(file);
        }

        let err = open_result.unwrap_err();
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(Error::WriteFileError(format!(
                "Fail to open file {p:?}: {err}"
            )));
        }

        tokio::fs::remove_file(p).await.map_err(|err| {
            Error::WriteFileError(format!(
                "There was already a file there and it is not deletable: {err}"
            ))
        })?;
        dbg!("Detect exist, removed");

        let open_result = tokio::fs::OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(p)
            .await;

        if let Err(err) = open_result {
            return Err(Error::WriteFileError(format!(
                "Fail to open file for download document, even after retry: {err}"
            )));
        }

        Ok(open_result.unwrap())
    }

    /// Download the possibly translated document. Downloaded document will store to the specific
    /// `output` path.
    ///
    /// Return downloaded file's path if success
    pub async fn download_document<O: AsRef<Path>>(
        &self,
        ident: &DocumentUploadResp,
        output: O,
    ) -> Result<PathBuf> {
        let url = self
            .endpoint
            .join(&format!("document/{}/result", ident.document_id))
            .unwrap();
        let form = [("document_key", ident.document_key.as_str())];
        let res = self
            .post(url)
            .form(&form)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NonExistDocument);
        }

        if res.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            return Err(Error::TranslationNotDone);
        }

        if !res.status().is_success() {
            return Self::extract_deepl_error(res).await;
        }

        let mut file = Self::open_file_to_write(output.as_ref()).await?;

        let mut stream = res.bytes_stream();

        #[inline]
        fn mapper<E: std::error::Error>(s: &'static str) -> Box<dyn FnOnce(E) -> Error> {
            Box::new(move |err: E| Error::WriteFileError(format!("{s}: {err}")))
        }

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(mapper("fail to download part of the document"))?;
            file.write_all(&chunk)
                .await
                .map_err(mapper("fail to write downloaded part into file"))?;
            file.sync_all()
                .await
                .map_err(mapper("fail to sync file content"))?;
        }

        Ok(output.as_ref().to_path_buf())
    }
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

    let raw_text = "Doubt thou the stars are fire. \
    Doubt that the sun doth move. \
    Doubt truth to be a liar. \
    But never doubt my love.";

    tokio::fs::write("./test.txt", &raw_text).await.unwrap();

    let test_file = PathBuf::from("./test.txt");
    let upload_option = UploadDocumentProp::builder()
        .target_lang(Lang::ZH)
        .file_path(&test_file)
        .build();
    let response = api.upload_document(upload_option).await.unwrap();
    let mut status = api.check_document_status(&response).await.unwrap();

    // wait for translation
    loop {
        if status.status.is_done() {
            break;
        }
        if let Some(msg) = status.error_message {
            println!("{}", msg);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        status = api.check_document_status(&response).await.unwrap();
        dbg!(&status);
    }

    let path = api
        .download_document(&response, "test_translated.txt")
        .await
        .unwrap();

    let content = tokio::fs::read_to_string(path).await.unwrap();
    let expect = "怀疑你的星星是火。怀疑太阳在动。怀疑真理是个骗子。但永远不要怀疑我的爱。";
    assert_eq!(content, expect);
}

#[tokio::test]
async fn test_upload_docx() {
    use pretty_assertions::assert_eq;
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key, false);

    let test_file = PathBuf::from("./asserts/example.docx");
    let upload_option = UploadDocumentProp::builder()
        .target_lang(Lang::ZH)
        .file_path(&test_file)
        .build();
    let response = api.upload_document(upload_option).await.unwrap();
    let mut status = api.check_document_status(&response).await.unwrap();

    // wait for translation
    loop {
        if status.status.is_done() {
            break;
        }
        if let Some(msg) = status.error_message {
            println!("{}", msg);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        status = api.check_document_status(&response).await.unwrap();
        dbg!(&status);
    }

    let path = api
        .download_document(&response, "translated.docx")
        .await
        .unwrap();
    let get = tokio::fs::read(&path).await.unwrap();
    let want = tokio::fs::read("./asserts/expected.docx").await.unwrap();
    assert_eq!(get, want);
}
