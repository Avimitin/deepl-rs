use serde::{Deserialize, Deserializer};
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

/// Available language code for source and target text
#[derive(Debug, PartialEq)]
pub enum Lang {
    BG,
    CS,
    DA,
    DE,
    EL,
    EN,
    ES,
    ET,
    FI,
    FR,
    HU,
    ID,
    IT,
    JA,
    LT,
    LV,
    NL,
    PL,
    PT,
    RO,
    RU,
    SK,
    SL,
    SV,
    TR,
    UK,
    ZH,
}

impl Lang {
    /// Convert literal to enum `Lang`
    ///
    /// # Error
    ///
    /// Return `Error::InvalidLang` when given language code is not in the support list.
    pub fn from(s: &str) -> Result<Self, Error> {
        let lang = match s {
            "BG" => Self::BG,
            "CS" => Self::CS,
            "DA" => Self::DA,
            "DE" => Self::DE,
            "EL" => Self::EL,
            "EN" => Self::EN,
            "ES" => Self::ES,
            "ET" => Self::ET,
            "FI" => Self::FI,
            "FR" => Self::FR,
            "HU" => Self::HU,
            "ID" => Self::ID,
            "IT" => Self::IT,
            "JA" => Self::JA,
            "LT" => Self::LT,
            "LV" => Self::LV,
            "NL" => Self::NL,
            "PL" => Self::PL,
            "PT" => Self::PT,
            "RO" => Self::RO,
            "RU" => Self::RU,
            "SK" => Self::SK,
            "SL" => Self::SL,
            "SV" => Self::SV,
            "TR" => Self::TR,
            "UK" => Self::UK,
            "ZH" => Self::ZH,
            _ => return Err(Error::InvalidLang(s.to_string())),
        };

        Ok(lang)
    }

    /// Return full language name for the code
    pub fn description(&self) -> String {
        match self {
            Self::BG => "Bulgarian".to_string(),
            Self::CS => "Czech".to_string(),
            Self::DA => "Danish".to_string(),
            Self::DE => "German".to_string(),
            Self::EL => "Greek".to_string(),
            Self::EN => "English".to_string(),
            Self::ES => "Spanish".to_string(),
            Self::ET => "Estonian".to_string(),
            Self::FI => "Finnish".to_string(),
            Self::FR => "French".to_string(),
            Self::HU => "Hungarian".to_string(),
            Self::ID => "Indonesian".to_string(),
            Self::IT => "Italian".to_string(),
            Self::JA => "Japanese".to_string(),
            Self::LT => "Lithuanian".to_string(),
            Self::LV => "Latvian".to_string(),
            Self::NL => "Dutch".to_string(),
            Self::PL => "Polish".to_string(),
            Self::PT => "Portuguese (all Portuguese varieties mixed)".to_string(),
            Self::RO => "Romanian".to_string(),
            Self::RU => "Russian".to_string(),
            Self::SK => "Slovak".to_string(),
            Self::SL => "Slovenian".to_string(),
            Self::SV => "Swedish".to_string(),
            Self::TR => "Turkish".to_string(),
            Self::UK => "Ukrainian".to_string(),
            Self::ZH => "Chinese".to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for Lang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lang = String::deserialize(deserializer)?;

        let lang = Lang::from(&lang).map_err(|_| {
            serde::de::Error::custom(
                format!("invalid language code {lang}. This is an internal issue with the lib, please open issue")
            )
        })?;

        Ok(lang)
    }
}

impl AsRef<str> for Lang {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::BG => "BG",
            Self::CS => "CS",
            Self::DA => "DA",
            Self::DE => "DE",
            Self::EL => "EL",
            Self::EN => "EN",
            Self::ES => "ES",
            Self::ET => "ET",
            Self::FI => "FI",
            Self::FR => "FR",
            Self::HU => "HU",
            Self::ID => "ID",
            Self::IT => "IT",
            Self::JA => "JA",
            Self::LT => "LT",
            Self::LV => "LV",
            Self::NL => "NL",
            Self::PL => "PL",
            Self::PT => "PT",
            Self::RO => "RO",
            Self::RU => "RU",
            Self::SK => "SK",
            Self::SL => "SL",
            Self::SV => "SV",
            Self::TR => "TR",
            Self::UK => "UK",
            Self::ZH => "ZH",
        }
    }
}

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
