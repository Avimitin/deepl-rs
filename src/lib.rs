use anyhow::Context;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

const TRANSLATE_TEXT_ENDPOINT: &str = "https://api-free.deepl.com/v2/translate";

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
    fn description(&self) -> String {
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

        let lang = match lang.as_str() {
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
            _ => return Err(
                serde::de::Error::custom(
                    // TODO: attach issue link
                    format!("invalid language code {lang}. This is an internal issue with the lib, please open issue")
                )
            ),
        };

        Ok(lang)
    }
}

impl AsRef<str> for Lang {
    fn as_ref(&self) -> &str {
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

#[derive(Deserialize)]
pub struct DeepLApiResponse {
    pub translations: Vec<SingleResult>,
}

#[derive(Deserialize)]
pub struct SingleResult {
    pub detected_source_language: Lang,
    pub text: String,
}

pub struct DeepLApi {
    client: reqwest::Client,
    key: String,
}

impl DeepLApi {
    pub fn new(key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            key: key.to_string(),
        }
    }

    pub async fn translate(
        &self,
        text: &str,
        translate_into: Lang,
    ) -> anyhow::Result<DeepLApiResponse> {
        let mut param = HashMap::new();
        param.insert("text", text);
        param.insert("target_lang", translate_into.as_ref());
        let response = self
            .client
            .post(TRANSLATE_TEXT_ENDPOINT)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.key))
            .form(&param)
            .send()
            .await
            .with_context(|| "fail to send request to DeepL Api")?
            .json::<DeepLApiResponse>()
            .await
            .with_context(|| "fail to transform DeepL response into `DeepLApiResponse` type")?;

        Ok(response)
    }
}

#[tokio::test]
async fn test_translator() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key);
    let response = api.translate("Hello World", Lang::ZH).await.unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(translated_results[0].text, "你好，世界");
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}
