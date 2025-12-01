use std::future::IntoFuture;

use crate::{
    endpoint::{Formality, Pollable, Result},
    impl_requester, Lang,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

/// Response from basic translation API
#[derive(Deserialize)]
pub struct TranslateTextResp {
    pub translations: Vec<Sentence>,
}

impl std::fmt::Display for TranslateTextResp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.translations
                .iter()
                .map(|sent| sent.text.to_string())
                .collect::<String>()
        )
    }
}

/// Translated result for a sentence
#[derive(Deserialize)]
pub struct Sentence {
    pub detected_source_language: Lang,
    pub text: String,
}

///
/// Sets whether the translation engine should respect the original formatting,
/// even if it would usually correct some aspects.
/// The formatting aspects affected by this setting include:
/// - Punctuation at the beginning and end of the sentence
/// - Upper/lower case at the beginning of the sentence
///
#[derive(Debug, Serialize)]
pub enum PreserveFormatting {
    #[serde(rename = "1")]
    Preserve,
    #[serde(rename = "0")]
    DontPreserve,
}

///
/// Sets whether the translation engine should first split the input into sentences
///
/// For applications that send one sentence per text parameter, it is advisable to set this to `None`,
/// in order to prevent the engine from splitting the sentence unintentionally.
/// Please note that newlines will split sentences. You should therefore clean files to avoid breaking sentences or set this to `PunctuationOnly`.
///
#[derive(Debug, Serialize)]
pub enum SplitSentences {
    /// Perform no splitting at all, whole input is treated as one sentence
    #[serde(rename = "0")]
    None,
    /// Split on punctuation and on newlines (default)
    #[serde(rename = "1")]
    PunctuationAndNewlines,
    /// Split on punctuation only, ignoring newlines
    #[serde(rename = "nonewlines")]
    PunctuationOnly,
}

///
/// Sets which kind of tags should be handled. Options currently available
///
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TagHandling {
    /// Enable XML tag handling
    /// see: <https://www.deepl.com/docs-api/xml>
    Xml,
    /// Enable HTML tag handling
    /// see: <https://www.deepl.com/docs-api/html>
    Html,
}

/// Sets the language model to use: allows to choose an improved "next-gen" model
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    /// Use classic lower-latency default model
    LatencyOptimized,
    /// Use improved higher-latency model
    QualityOptimized,
    /// Use improved higher-latency model, but fallback to default model if not available for the selected language
    PreferQualityOptimized,
}

impl_requester! {
    TranslateRequester {
        @required{
            text: Vec<String>,
            target_lang: Lang,
        };
        @optional{
            context: String,
            source_lang: Lang,
            split_sentences: SplitSentences,
            preserve_formatting: PreserveFormatting,
            formality: Formality,
            glossary_id: String,
            tag_handling: TagHandling,
            model_type: ModelType,
            non_splitting_tags: Vec<String>,
            splitting_tags: Vec<String>,
            ignore_tags: Vec<String>,
        };
    } -> Result<TranslateTextResp, Error>;
}

impl<'a> IntoFuture for TranslateRequester<'a> {
    type Output = Result<TranslateTextResp>;
    type IntoFuture = Pollable<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        self.send()
    }
}

impl<'a> IntoFuture for &mut TranslateRequester<'a> {
    type Output = Result<TranslateTextResp>;
    type IntoFuture = Pollable<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        self.send()
    }
}

impl<'a> TranslateRequester<'a> {
    fn send(&self) -> Pollable<'a, Result<TranslateTextResp>> {
        let client = self.client.clone();
        let obj = json!(self);

        let fut = async move {
            let response = client
                .post(client.inner.endpoint.join("translate").unwrap())
                .json(&obj)
                .send()
                .await
                .map_err(|err| Error::RequestFail(err.to_string()))?;

            if !response.status().is_success() {
                return super::extract_deepl_error(response).await;
            }

            let response: TranslateTextResp = response.json().await.map_err(|err| {
                Error::InvalidResponse(format!("convert json bytes to Rust type: {err}"))
            })?;

            Ok(response)
        };

        Box::pin(fut)
    }
}

pub trait ToTranslatable {
    fn to_translatable(&self) -> Vec<String>;
}

impl ToTranslatable for String {
    fn to_translatable(&self) -> Vec<String> {
        vec![self.to_owned()]
    }
}

impl ToTranslatable for &str {
    fn to_translatable(&self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl ToTranslatable for Vec<String> {
    fn to_translatable(&self) -> Vec<String> {
        self.clone()
    }
}

impl ToTranslatable for &[String] {
    fn to_translatable(&self) -> Vec<String> {
        self.to_vec()
    }
}

impl ToTranslatable for &[&str] {
    fn to_translatable(&self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

impl DeepLApi {
    /// Translate the given text with specific target language.
    ///
    /// # Error
    ///
    /// Return [`Error`] if the http request fail
    ///
    /// # Example
    ///
    /// * Simple translation
    ///
    /// ```rust
    /// use deepl::{DeepLApi, Lang};
    ///
    /// let key = std::env::var("DEEPL_API_KEY").unwrap();
    /// let deepl = DeepLApi::with(&key).new();
    ///
    /// let response = deepl.translate_text("Hello World", Lang::ZH).await.unwrap();
    /// assert!(!response.translations.is_empty());
    /// ```
    ///
    /// * Translation with XML tag enabled
    ///
    /// ```rust
    /// use deepl::{DeepLApi, Lang};
    ///
    /// let key = std::env::var("DEEPL_API_KEY").unwrap();
    /// let deepl = DeepLApi::with(&key).new();
    ///
    /// let str = "Hello World <keep>This will stay exactly the way it was</keep>";
    /// let response = deepl
    ///     .translate_text(str, Lang::DE)
    ///     .source_lang(Lang::EN)
    ///     .ignore_tags(vec!["keep".to_owned()])
    ///     .tag_handling(TagHandling::Xml)
    ///     .await
    ///     .unwrap();
    ///
    /// let translated_results = response.translations;
    /// let should = "Hallo Welt <keep>This will stay exactly the way it was</keep>";
    /// assert_eq!(translated_results[0].text, should);
    /// ```
    pub fn translate_text(
        &self,
        input: impl ToTranslatable,
        target_lang: Lang,
    ) -> TranslateRequester<'_> {
        TranslateRequester::new(self, input.to_translatable(), target_lang)
    }
}

#[tokio::test]
async fn test_translate_text() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::with(&key).new();
    let response = api.translate_text("Hello World", Lang::ZH).await.unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(translated_results[0].text, "你好，世界");
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}

#[tokio::test]
async fn test_advanced_translate() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::with(&key).new();

    let response = api.translate_text(
            "Hello World <keep additionalarg=\"test0\">This will stay exactly the way it was</keep>",
            Lang::DE
        )
        .source_lang(Lang::EN)
        .ignore_tags(vec!["keep".to_string()])
        .tag_handling(TagHandling::Xml)
        .await
        .unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(
        translated_results[0].text,
        "Hallo Welt <keep additionalarg=\"test0\">This will stay exactly the way it was</keep>"
    );
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}

#[tokio::test]
async fn test_models() {
    let api = DeepLApi::with(&std::env::var("DEEPL_API_KEY").unwrap()).new();

    // whatever model is used, the translation should happen, and it can differ slightly
    for model_type in [
        ModelType::LatencyOptimized,
        ModelType::QualityOptimized,
        ModelType::QualityOptimized,
    ] {
        let response = api
            .translate_text("No te muevas, pringao", Lang::EN)
            .source_lang(Lang::ES)
            .model_type(model_type)
            .await
            .unwrap();

        assert!(response
            .translations
            .first()
            .expect("should be a translation")
            .text
            .contains("Don't move")); // the last word can differ depending on the model chosen
    }
}

#[tokio::test]
async fn test_advanced_translator_html() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::with(&key).new();

    let response = api
        .translate_text(
            "Hello World <keep translate=\"no\">This will stay exactly the way it was</keep>",
            Lang::DE,
        )
        .tag_handling(TagHandling::Html)
        .await
        .unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(
        translated_results[0].text,
        "Hallo Welt <keep translate=\"no\">This will stay exactly the way it was</keep>"
    );
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}

#[tokio::test]
async fn test_formality() {
    let api = DeepLApi::with(&std::env::var("DEEPL_API_KEY").unwrap()).new();

    // can specify a formality
    let text = "How are you?";
    let src = Lang::EN;
    let trg = Lang::ES;
    let more = Formality::More;

    let response = api
        .translate_text(text, trg)
        .source_lang(src)
        .formality(more)
        .await
        .unwrap();
    assert!(!response.translations.is_empty());

    // response ok, despite target lang not supporting formality
    let text = "¿Cómo estás?";
    let src = Lang::ES;
    let trg = Lang::EN_US;
    let less = Formality::PreferLess;

    let response = api
        .translate_text(text, trg)
        .source_lang(src)
        .formality(less)
        .await
        .unwrap();
    assert!(!response.translations.is_empty());
}
