//! # deepl-rs
//!
//! Deepl-rs is a simple library for making requests to the DeepL API endpoint easier.
//! And it also provides types wrapping to guarantee runtime safety.
//!
//! This is still a **WORK IN PROGRESS** library, please open an issue on GitHub to request
//! features. Be aware breaking changes will be released frequently.
//!
//! # Usage
//!
//! ```rust
//! use deepl::DeepLApi;
//!
//! let key = std::env::var("DEEPL_API_KEY").unwrap();
//! let api = DeepLApi::with(&key).new();
//! let response = api.translate_text("Hello World", Lang::ZH).await.unwrap();
//!
//! assert!(!response.translations.is_empty());
//! ```
//!
//! See [`DeepLApi`] for detailed usage.
//!
//! # License
//!
//! This project is licensed under MIT license.
//!

mod endpoint;
mod lang;

use std::sync::Arc;

//- Type Re-exporting
pub use endpoint::{
    document::{DocumentStatusResp, DocumentTranslateStatus, UploadDocumentResp},
    translate::{TagHandling, TranslateTextResp},
    usage::UsageResponse,
    Error, Formality,
};
pub use lang::Lang;
pub use reqwest;
//-

/// A struct that contains necessary data for runtime. Data is stored in
/// [`Arc`], so it is cheap to clone in your App's code.
///
/// # Example
///
/// ```
/// // simple API creation
/// let deepl = DeepLApi::with("Your DeepL Key").new();
///
/// // **OR** customize it
/// let duration = std::time::Duration::from_secs(30);
/// let client = reqwest::Client::builder()
///         .timeout(duration)
///         .build()
///         .unwrap();
///
/// // use the pro version API, and a custom client with
/// // 30 secs timeout
/// let deepl = DeepLApi::with("Your DeepL Key")
///                 .is_pro(true)
///                 .client(client)
///                 .new();
/// ```
#[derive(Debug, Clone)]
pub struct DeepLApi {
    inner: Arc<DeepLApiInner>,
}

/// The inner instance which actually holds data
#[derive(Debug)]
struct DeepLApiInner {
    client: reqwest::Client,
    key: String,
    endpoint: reqwest::Url,
}

impl DeepLApi {
    /// Create a new api instance with auth key.
    pub fn with(key: &str) -> DeepLApiBuilder {
        DeepLApiBuilder::init(key.to_string())
    }

    fn post(&self, url: reqwest::Url) -> reqwest::RequestBuilder {
        self.inner
            .client
            .post(url)
            .header("Authorization", &self.inner.key)
    }

    fn get_endpoint(&self, route: &str) -> reqwest::Url {
        self.inner.endpoint.join(route).unwrap()
    }
}

/// The builder struct. **DO NOT USE IT IN YOUR APPS**
pub struct DeepLApiBuilder {
    is_pro: bool,
    client: Option<reqwest::Client>,
    key: String,
}

impl DeepLApiBuilder {
    fn init(key: String) -> Self {
        Self {
            key,
            is_pro: false,
            client: None,
        }
    }

    /// Set the a user defined [`reqwest::Client`]
    pub fn client(&mut self, c: reqwest::Client) -> &mut Self {
        self.client = Some(c);
        self
    }

    /// Set if you want to use the pro version DeepL Api
    pub fn is_pro(&mut self, is_pro: bool) -> &mut Self {
        self.is_pro = is_pro;
        self
    }

    /// Create a new instance of the DeepLApi
    pub fn new(&self) -> DeepLApi {
        let client = self.client.clone().unwrap_or_else(reqwest::Client::new);
        let endpoint = if self.is_pro {
            "https://api.deepl.com/v2/"
        } else {
            "https://api-free.deepl.com/v2/"
        };

        let inner = DeepLApiInner {
            key: format!("DeepL-Auth-Key {}", self.key),
            client,
            endpoint: reqwest::Url::parse(endpoint).unwrap(),
        };

        DeepLApi {
            inner: Arc::new(inner),
        }
    }
}
