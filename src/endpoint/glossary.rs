use crate::{
    endpoint::{Error, Result},
    DeepLApi, Lang,
};
use core::future::IntoFuture;
use typed_builder::TypedBuilder;

use super::Pollable;

#[derive(Debug, TypedBuilder)]
#[builder(build_method(name = send))]
pub struct CreateGlossary<'a> {
    client: &'a DeepLApi,

    name: String,

    #[builder(setter(transform = |a: impl ToString, b: Lang| (a.to_string(), b)))]
    source: (String, Lang),

    #[builder(setter(transform = |a: impl ToString, b: Lang| (a.to_string(), b)))]
    target: (String, Lang),

    #[builder(default = EntriesFormat::TSV)]
    format: EntriesFormat,
}

type CreateGlossaryBuilderStart<'a> =
    CreateGlossaryBuilder<'a, ((&'a DeepLApi,), (String,), (), (), ())>;

impl<'a> IntoFuture for CreateGlossary<'a> {
    type Output = Result<CreateGloassaryResp>;
    type IntoFuture = Pollable<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let client = self.client.clone();
        let fields = CreateGlossaryRequestParam::from(self);
        let fut = async move {
            let resp = client
                        .post(client.inner.endpoint.join("glossary").unwrap())
                        .json(&fields)
                        .send()
                        .await
                        .map_err(|err| Error::RequestFail(err.to_string()))?
                        .json::<CreateGloassaryResp>()
                        .await
                        .expect("Unmathched response to CreateGloassaryResp, please open issue on https://github.com/Avimitin/deepl.");
            Ok(resp)
        };

        Box::pin(fut)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateGloassaryResp {
    gloassary_id: String,
    name: String,
    ready: bool,
    source_lang: String,
    target_lang: String,
    creation_time: String,
    entry_count: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CreateGlossaryRequestParam {
    name: String,
    source_lang: String,
    target_lang: String,
    entries: String,
    entries_format: String,
}

impl<'a> From<CreateGlossary<'a>> for CreateGlossaryRequestParam {
    fn from(value: CreateGlossary<'a>) -> Self {
        CreateGlossaryRequestParam {
            name: value.name,
            source_lang: value.source.1.to_string(),
            target_lang: value.target.1.to_string(),
            entries: match value.format {
                EntriesFormat::TSV => format!("{}\t{}", value.source.0, value.target.0),
                EntriesFormat::CSV => format!("{}:{}", value.source.0, value.target.0),
            },
            entries_format: value.format.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum EntriesFormat {
    TSV,
    CSV,
}

impl ToString for EntriesFormat {
    fn to_string(&self) -> String {
        match self {
            EntriesFormat::TSV => "tsv".to_string(),
            EntriesFormat::CSV => "csv".to_string(),
        }
    }
}

impl DeepLApi {
    /// API for endpoint: https://www.deepl.com/de/docs-api/glossaries/create-glossary.
    /// The function for creating a glossary returns a JSON object containing the
    /// ID of the newly created glossary and a boolean flag that indicates if the
    /// created glossary can already be used in translate requests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::{glossary::EntriesFormat, DeepLApi, Lang};
    ///
    /// let key = std::env::var("DEEPL_API_KEY").unwrap();
    /// let deepl = DeepLApi::with(&key).new();
    ///
    /// let _: CreateGloassaryResp = deepl
    ///     .create_glossary("My Gloassary")
    ///     .source("Hello", Lang::EN)
    ///     .target("Guten Tag", Lang::DE)
    ///     .format(EntriesFormat::CSV) // This field is optional, we will use TSV as default.
    ///     .send()
    ///     .await
    ///     .unwrap();
    /// ```
    pub fn create_glossary(&self, name: impl ToString) -> CreateGlossaryBuilderStart {
        CreateGlossary::builder()
            .client(self)
            .name(name.to_string())
    }
}

#[tokio::test]
async fn test_create_gloassary() {
    use crate::{glossary::EntriesFormat, DeepLApi, Lang};

    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let deepl = DeepLApi::with(&key).new();

    let _: CreateGloassaryResp = deepl
        .create_glossary("My Gloassary")
        .source("Hello", Lang::EN)
        .target("Guten Tag", Lang::DE)
        .format(EntriesFormat::CSV) // This field is optional, we will use TSV as default.
        .send()
        .await
        .unwrap();
}
