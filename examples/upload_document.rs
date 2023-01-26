use deepl::{DeepLApi, Lang};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::with(&key).new();

    let raw_text = "Doubt thou the stars are fire. \
    Doubt that the sun doth move. \
    Doubt truth to be a liar. \
    But never doubt my love.";

    tokio::fs::write("./test.txt", &raw_text).await.unwrap();

    let test_file = PathBuf::from("./test.txt");
    let response = api.upload_document(&test_file, Lang::ZH).await.unwrap();
    let mut status = api.check_document_status(&response).await.unwrap();

    // wait for translation
    loop {
        if status.status.is_done() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        status = api.check_document_status(&response).await.unwrap();
    }

    let path = api
        .download_document(&response, "test_translated.txt")
        .await
        .unwrap();

    let content = tokio::fs::read_to_string(path).await.unwrap();
    let expect = "怀疑你的星星是火。怀疑太阳在动。怀疑真理是个骗子。但永远不要怀疑我的爱。";
    assert_eq!(content, expect);
}
