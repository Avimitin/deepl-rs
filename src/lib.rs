use anyhow::Context;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

const TRANSLATE_TEXT_ENDPOINT: &str = "https://api-free.deepl.com/v2/translate";

#[derive(Debug, PartialEq)]
pub enum Lang {
    EN,
    ZH,
}

impl<'de> Deserialize<'de> for Lang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lang = String::deserialize(deserializer)?;

        let lang = match lang.as_str() {
            "EN" => Self::EN,
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
            Self::EN => "EN",
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
