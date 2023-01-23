use crate::{DeepLApi, Lang};

pub mod document;
pub mod translate;
pub mod usage;

impl DeepLApi {
    pub fn translate_text(&self, text: &str, target_lang: Lang) -> translate::TranslateRequester {
        translate::TranslateRequester::new(self, text, target_lang)
    }
}
