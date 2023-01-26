use deepl::TagHandling;
use deepl::{DeepLApi, Lang};

#[tokio::main]
async fn main() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let deepl = DeepLApi::with(&key).new();

    let translated = deepl.translate_text("Hello World", Lang::DE).await.unwrap();
    println!("Translated text: ");
    println!("{translated}");

    let api = DeepLApi::with("YOUR AUTH KEY").new();
    let str = "Hello World <keep>This will stay exactly the way it was</keep>";
    let response = api
        .translate_text(str, Lang::DE)
        .source_lang(Lang::EN)
        .ignore_tags(vec!["keep".to_owned()])
        .tag_handling(TagHandling::Xml)
        .await
        .unwrap();

    let translated_results = response.translations;
    let should = "Hallo Welt <keep>This will stay exactly the way it was</keep>";
    assert_eq!(translated_results[0].text, should);
}
