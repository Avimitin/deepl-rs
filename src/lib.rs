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

pub use endpoint::{
    document::{DocumentStatusResp, DocumentTranslateStatus, UploadDocumentResp},
    translate::{TagHandling, TranslateTextResp},
    usage::UsageResponse,
    Error, Formality,
};
pub use lang::Lang;
pub use reqwest;

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
#[derive(Debug)]
pub struct DeepLApi {
    client: reqwest::Client,
    key: String,
    endpoint: reqwest::Url,
}

impl DeepLApi {
    /// Create a new api instance with auth key. If you are paid user, pass `true` into the second
    /// parameter.
    pub fn new(key: &str) -> DeepLApiBuilder {
        DeepLApiBuilder::new(key.to_string())
    }

    fn post(&self, url: reqwest::Url) -> reqwest::RequestBuilder {
        self.client.post(url).header("Authorization", &self.key)
    }
}

pub struct DeepLApiBuilder {
    is_pro: bool,
    client: Option<reqwest::Client>,
    key: String,
}

impl DeepLApiBuilder {
    fn new(key: String) -> Self {
        Self {
            key,
            is_pro: false,
            client: None,
        }
    }

    pub fn client(&mut self, c: reqwest::Client) -> &mut Self {
        self.client = Some(c);
        self
    }

    pub fn is_pro(&mut self, is_pro: bool) -> &mut Self {
        self.is_pro = is_pro;
        self
    }

    pub fn build(&self) -> DeepLApi {
        let client = self.client.clone().unwrap_or_else(reqwest::Client::new);
        let endpoint = if self.is_pro {
            "https://api.deepl.com/v2/"
        } else {
            "https://api-free.deepl.com/v2/"
        };

        DeepLApi {
            key: format!("DeepL-Auth-Key {}", self.key),
            client,
            endpoint: reqwest::Url::parse(endpoint).unwrap(),
        }
    }
}
