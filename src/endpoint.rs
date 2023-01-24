use crate::{DeepLApi, Lang, TagHandling};

pub mod document;
pub mod translate;
pub mod usage;

impl DeepLApi {
    pub fn translate_text(&self, text: &str, target_lang: Lang) -> translate::TranslateRequester {
        translate::TranslateRequester::new(self, text.to_string(), target_lang)
    }
}

#[tokio::test]
async fn test_translate_text() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key, false);
    let response = api.translate_text("Hello World", Lang::ZH).await.unwrap();

    assert!(!response.translations.is_empty());

    let translated_results = response.translations;
    assert_eq!(translated_results[0].text, "你好，世界");
    assert_eq!(translated_results[0].detected_source_language, Lang::EN);
}

#[tokio::test]
async fn test_advanced_translate() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::new(&key, false);

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
