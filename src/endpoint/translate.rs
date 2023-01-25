use std::collections::HashMap;

use crate::{
    endpoint::{Pollable, Result, ToPollable},
    impl_requester, Lang,
};

use serde::Deserialize;

/// Response from basic translation API
#[derive(Deserialize)]
pub struct TranslateTextResp {
    pub translations: Vec<Sentence>,
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
pub enum PreserveFormatting {
    Preserve,
    DontPreserve,
}

impl AsRef<str> for PreserveFormatting {
    fn as_ref(&self) -> &str {
        match self {
            PreserveFormatting::Preserve => "1",
            PreserveFormatting::DontPreserve => "0",
        }
    }
}

///
/// Sets whether the translation engine should first split the input into sentences
///
/// For applications that send one sentence per text parameter, it is advisable to set this to `None`,
/// in order to prevent the engine from splitting the sentence unintentionally.
/// Please note that newlines will split sentences. You should therefore clean files to avoid breaking sentences or set this to `PunctuationOnly`.
///
pub enum SplitSentences {
    /// Perform no splitting at all, whole input is treated as one sentence
    None,
    /// Split on punctuation and on newlines (default)
    PunctuationAndNewlines,
    /// Split on punctuation only, ignoring newlines
    PunctuationOnly,
}

impl AsRef<str> for SplitSentences {
    fn as_ref(&self) -> &str {
        match self {
            SplitSentences::None => "0",
            SplitSentences::PunctuationAndNewlines => "1",
            SplitSentences::PunctuationOnly => "nonewlines",
        }
    }
}

///
/// Sets which kind of tags should be handled. Options currently available
///
pub enum TagHandling {
    /// Enable XML tag handling
    /// see: <https://www.deepl.com/docs-api/xml>
    Xml,
    /// Enable HTML tag handling
    /// see: <https://www.deepl.com/docs-api/html>
    Html,
}

impl AsRef<str> for TagHandling {
    fn as_ref(&self) -> &str {
        match self {
            TagHandling::Xml => "xml",
            TagHandling::Html => "html",
        }
    }
}

impl_requester! {
    Translate {
        @must{
            text: String,
            target_lang: Lang,
        };
        @optional{
            source_lang: Lang,
            split_sentences: SplitSentences,
            preserve_formatting: PreserveFormatting,
            glossary_id: String,
            tag_handling: TagHandling,
            non_splitting_tags: Vec<String>,
            splitting_tags: Vec<String>,
            ignore_tags: Vec<String>,
        };
    } -> Result<TranslateTextResp, Error>;
}

impl<'a> ToPollable<Result<TranslateTextResp>> for TranslateRequester<'a> {
    fn to_pollable(&mut self) -> Pollable<Result<TranslateTextResp>> {
        Box::pin(self.send())
    }
}

impl<'a> TranslateRequester<'a> {
    async fn send(&self) -> Result<TranslateTextResp, Error> {
        let mut param = HashMap::new();
        param.insert("text", self.text.as_str());

        if let Some(ref la) = self.source_lang {
            param.insert("source_lang", la.as_ref());
        }
        param.insert("target_lang", self.target_lang.as_ref());
        if let Some(ref ss) = self.split_sentences {
            param.insert("split_sentences", ss.as_ref());
        }
        if let Some(ref pf) = self.preserve_formatting {
            param.insert("preserve_formatting", pf.as_ref());
        }
        if let Some(ref id) = self.glossary_id {
            param.insert("glossary_id", id);
        }
        if let Some(ref th) = self.tag_handling {
            param.insert("tag_handling", th.as_ref());
        }

        let ns_tags: String;
        if let Some(tags) = &self.non_splitting_tags {
            if !tags.is_empty() {
                ns_tags = tags.join(",");
                param.insert("non_splitting_tags", &ns_tags);
            }
        }

        let sp_tags: String;
        if let Some(tags) = &self.splitting_tags {
            if !tags.is_empty() {
                sp_tags = tags.join(",");
                param.insert("splitting_tags", &sp_tags);
            }
        }

        let ig_tags: String;
        if let Some(tags) = &self.ignore_tags {
            if !tags.is_empty() {
                ig_tags = tags.join(",");
                param.insert("ignore_tags", &ig_tags);
            }
        }

        let response = self
            .client
            .post(self.client.endpoint.join("translate").unwrap())
            .form(&param)
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
    }
}

impl DeepLApi {
    /// Translate the given text using the given text translation settings.
    ///
    /// # Error
    ///
    /// Return [`crates::Error`] if the http request fail
    ///
    /// # Example
    ///
    /// * Simple translation
    ///
    /// ```rust
    /// use deepl::{DeepLApi, Lang};
    ///
    /// let api = DeepLApi::new("YOUR AUTH KEY", false);
    ///
    /// // Translate "Hello World" to Chinese
    /// let response = api.translate_text("Hello World", Lang::ZH).await.unwrap();
    ///
    /// assert!(!response.translations.is_empty());
    /// ```
    ///
    /// * Translation with XML tag enabled
    ///
    /// ```rust
    /// use deepl::{DeepLApi, Lang};
    ///
    /// let api = DeepLApi::new("YOUR AUTH KEY", false);
    /// let str = "Hello World <keep>This will stay exactly the way it was</keep>";
    /// let response = api.translate_text(str, Lang::DE)
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
    pub fn translate_text(&self, text: impl ToString, target_lang: Lang) -> TranslateRequester {
        TranslateRequester::new(self, text.to_string(), target_lang)
    }
}

#[tokio::test]
async fn test_translate_text() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key).build();
    let response = api.translate_text("Hello World", Lang::ZH).await.unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(translated_results[0].text, "你好，世界");
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}

#[tokio::test]
async fn test_advanced_translate() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key).build();

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
async fn test_advanced_translator_html() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key).build();

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
