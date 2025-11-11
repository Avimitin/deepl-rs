use crate::{
    endpoint::{Error, Result, REPO_URL},
    DeepLApi,
};
use core::fmt;
use core::future::IntoFuture;
use std::borrow::Borrow;
use std::collections::HashMap;
use typed_builder::TypedBuilder;

use super::Pollable;

mod languages;

pub use languages::GlossaryLanguage;

#[derive(Debug, TypedBuilder)]
#[builder(build_method(name = send))]
pub struct CreateGlossary<'a> {
    client: &'a DeepLApi,

    name: String,

    source_lang: GlossaryLanguage,
    target_lang: GlossaryLanguage,

    #[builder(setter(prefix = "__"))]
    entries: Vec<(String, String)>,

    #[builder(default = EntriesFormat::TSV)]
    format: EntriesFormat,
}

#[allow(non_camel_case_types)]
#[allow(clippy::type_complexity)]
impl<'a, _c, _n, _s, _t, _f> CreateGlossaryBuilder<'a, (_c, _n, _s, _t, (), _f)> {
    /// The entries of the glossary.
    ///
    /// Entries parameter should be able to yield type (String, String).
    ///
    /// # Example
    /// ```rust
    /// let my_entries = vec![("Hello", "Guten Tag"), ("Bye", "Auf Wiedersehen")];
    /// // Pass in a HashMap is also okay.
    /// // let my_entries = HashMap::from([("Hello", "Guten Tag"), ("Bye", "Auf Wiedersehen")]);
    /// let resp = deepl
    ///     .create_glossary("My Glossary")
    ///     .source_lang(GlossaryLanguage::En)
    ///     .target_lang(GlossaryLanguage::De)
    ///     .entries(&my_entries)
    ///     .format(EntriesFormat::CSV) // This field is optional, we will use TSV as default.
    ///     .send()
    ///     .await
    ///     .unwrap();
    /// assert_eq!(resp.name, "My Glossary");
    /// ```
    pub fn entries<S, T, B, I>(
        self,
        iter: I,
    ) -> CreateGlossaryBuilder<'a, (_c, _n, _s, _t, (Vec<(String, String)>,), _f)>
    where
        S: ToString,
        T: ToString,
        B: Borrow<(S, T)>,
        I: IntoIterator<Item = B>,
    {
        let entries = iter
            .into_iter()
            .map(|t| (t.borrow().0.to_string(), t.borrow().1.to_string()))
            .collect();
        let (client, name, source_lang, target_lang, (), format) = self.fields;
        CreateGlossaryBuilder {
            fields: (client, name, source_lang, target_lang, (entries,), format),
            phantom: self.phantom,
        }
    }
}

type CreateGlossaryBuilderStart<'a> =
    CreateGlossaryBuilder<'a, ((&'a DeepLApi,), (String,), (), (), (), ())>;

impl<'a> IntoFuture for CreateGlossary<'a> {
    type Output = Result<GlossaryResp>;
    type IntoFuture = Pollable<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let client = self.client.clone();
        let fields = CreateGlossaryRequestParam::from(self);
        let fut = async move {
            let resp = client
                .post(client.get_endpoint("glossaries"))
                .json(&fields)
                .send()
                .await
                .map_err(|err| Error::RequestFail(err.to_string()))?;
            if !resp.status().is_success() {
                return super::extract_deepl_error(resp).await;
            }
            resp.json::<GlossaryResp>().await.map_err(|e| {
                Error::InvalidResponse(format!("convert json bytes to Rust type: {e}"))
            })
        };

        Box::pin(fut)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GlossaryResp {
    /// A unique ID assigned to a glossary.
    pub glossary_id: String,
    /// Name associated with the glossary.
    pub name: String,
    /// Indicates if the newly created glossary can already be used in translate requests.
    /// If the created glossary is not yet ready, you have to wait and check the ready status
    /// of the glossary before using it in a translate request.
    pub ready: bool,
    /// The language in which the source texts in the glossary are specified.
    pub source_lang: GlossaryLanguage,
    /// The language in which the target texts in the glossary are specified.
    pub target_lang: GlossaryLanguage,
    /// The creation time of the glossary in the ISO 8601-1:2019 format (e.g.: 2021-08-03T14:16:18.329Z).
    pub creation_time: String,
    /// The number of entries in the glossary.
    pub entry_count: u64,
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
            source_lang: value.source_lang.to_string().to_lowercase(),
            target_lang: value.target_lang.to_string().to_lowercase(),
            entries: match value.format {
                EntriesFormat::TSV => value
                    .entries
                    .iter()
                    .map(|(x, y)| format!("{x}\t{y}"))
                    .collect::<Vec<String>>()
                    .join("\n"),
                EntriesFormat::CSV => value
                    .entries
                    .iter()
                    .map(|(x, y)| format!("{x},{y}"))
                    .collect::<Vec<String>>()
                    .join("\n"),
            },
            entries_format: value.format.to_string(),
        }
    }
}

/// Format used for glossary entries.
#[derive(Debug)]
pub enum EntriesFormat {
    /// Tab separated values (TSV)
    TSV,
    /// Comma separated values (CSV)
    CSV,
}

impl fmt::Display for EntriesFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TSV => f.write_str("tsv"),
            Self::CSV => f.write_str("csv"),
        }
    }
}

/// Represents a response to `v2/glossary-language-pairs`.
#[derive(Debug, serde::Deserialize)]
struct GlossaryLanguagePairsResponse {
    /// Supported languages.
    supported_languages: Vec<GlossaryLanguagePair>,
}

/// A glossary language pair.
#[derive(Debug, serde::Deserialize)]
pub struct GlossaryLanguagePair {
    /// Source language
    pub source_lang: GlossaryLanguage,
    /// Target language
    pub target_lang: GlossaryLanguage,
}

impl DeepLApi {
    /// API for endpoint: <https://www.deepl.com/de/docs-api/glossaries/create-glossary>.
    /// The function for creating a glossary returns a JSON object containing the
    /// ID of the newly created glossary and a boolean flag that indicates if the
    /// created glossary can already be used in translate requests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::{glossary::{EntriesFormat, GlossaryLanguage}, DeepLApi};
    ///
    /// let key = std::env::var("DEEPL_API_KEY").unwrap();
    /// let deepl = DeepLApi::with(&key).new();
    ///
    /// let _: CreateGlossaryResp = deepl
    ///     .create_glossary("My Glossary")
    ///     .source("Hello", GlossaryLanguage::En)
    ///     .target("Guten Tag", GlossaryLanguage::De)
    ///     .format(EntriesFormat::CSV) // This field is optional, we will use TSV as default.
    ///     .send()
    ///     .await
    ///     .unwrap();
    /// ```
    pub fn create_glossary(&self, name: impl ToString) -> CreateGlossaryBuilderStart<'_> {
        CreateGlossary::builder()
            .client(self)
            .name(name.to_string())
    }

    /// List all glossaries and their meta-information, but not the glossary entries.
    pub async fn list_all_glossaries(&self) -> Result<Vec<GlossaryResp>> {
        let resp = self
            .get(self.get_endpoint("glossaries"))
            .send()
            .await
            .map_err(|e| Error::RequestFail(e.to_string()))?;
        if !resp.status().is_success() {
            return super::extract_deepl_error(resp).await;
        }
        resp.json::<HashMap<String, Vec<GlossaryResp>>>()
            .await
            .map_err(|err| Error::RequestFail(format!("Unexpected error when requesting list_all_glossaries, please open issue on {REPO_URL}: {err}")))?
            .remove("glossaries")
            .ok_or(Error::RequestFail(format!("Unable to find key glossaries in response, please open issue on {REPO_URL}")))
    }

    /// Retrieve meta information for a single glossary, omitting the glossary entries.
    /// Require a unique ID assigned to the glossary.
    pub async fn retrieve_glossary_details(&self, id: impl ToString) -> Result<GlossaryResp> {
        let resp = self
            .get(self.get_endpoint(&format!("glossaries/{}", id.to_string())))
            .send()
            .await
            .map_err(|e| Error::RequestFail(e.to_string()))?;
        if !resp.status().is_success() {
            return super::extract_deepl_error(resp).await;
        }
        resp.json::<GlossaryResp>()
            .await
            .map_err(|e| Error::InvalidResponse(format!("convert json bytes to Rust type: {e}")))
    }

    /// Deletes the specified glossary.
    pub async fn delete_glossary(&self, id: impl ToString) -> Result<()> {
        self.del(self.get_endpoint(&format!("glossaries/{}", id.to_string())))
            .send()
            .await
            .map_err(|e| Error::RequestFail(e.to_string()))
            .map(|_| ())
    }

    /// List the entries of a single glossary in the format specified by the Accept header.
    /// Currently, support TSV(tab separated value) only.
    pub async fn retrieve_glossary_entries(
        &self,
        id: impl ToString,
    ) -> Result<Vec<(String, String)>> {
        Ok(self
            .get(self.get_endpoint(&format!("glossaries/{}/entries", id.to_string())))
            .header("Accept", "text/tab-separated-values")
            .send()
            .await
            .map_err(|e| Error::RequestFail(e.to_string()))?
            .text()
            .await
            .map(|resp| {
                resp.split("\n")
                    .map(|line| {
                        let mut pair = line.split("\t");
                        (
                            pair.next().unwrap().to_string(),
                            pair.next().unwrap().to_string(),
                        )
                    })
                    .collect()
            })
            .map_err(|err| {
                Error::RequestFail(format!("fail to retrieve glossary entries: {err}"))
            }))?
    }

    /// Retrieve the list of language pairs supported by the glossary feature.
    pub async fn list_glossary_language_pairs(&self) -> Result<Vec<GlossaryLanguagePair>> {
        let resp = self
            .get(self.get_endpoint("glossary-language-pairs"))
            .send()
            .await
            .map_err(|e| Error::RequestFail(e.to_string()))?;
        if !resp.status().is_success() {
            return super::extract_deepl_error(resp).await;
        }
        resp.json::<GlossaryLanguagePairsResponse>()
            .await
            .map(|resp| resp.supported_languages)
            .map_err(|err| {
                Error::InvalidResponse(format!("convert json bytes to Rust type: {err}"))
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_list_glossary_languages() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let deepl = DeepLApi::with(&key).new();

        let pairs = deepl.list_glossary_language_pairs().await.unwrap();
        assert!(!pairs.is_empty());
    }

    #[tokio::test]
    async fn test_glossary_api() {
        let key = std::env::var("DEEPL_API_KEY").unwrap();
        let deepl = DeepLApi::with(&key).new();

        let my_entries = vec![("Hello", "Guten Tag"), ("Bye", "Auf Wiedersehen")];
        // let my_entries = HashMap::from([("Hello", "Guten Tag"), ("Bye", "Auf Wiedersehen")]);
        let resp = deepl
            .create_glossary("My Glossary")
            .source_lang(GlossaryLanguage::En)
            .target_lang(GlossaryLanguage::De)
            .entries(&my_entries)
            .format(EntriesFormat::CSV) // This field is optional, we will use TSV as default.
            .send()
            .await
            .unwrap();
        assert_eq!(resp.name, "My Glossary");

        let all = deepl.list_all_glossaries().await.unwrap();
        assert_ne!(all.len(), 0);

        let detail = deepl
            .retrieve_glossary_details(&resp.glossary_id)
            .await
            .unwrap();

        assert_eq!(detail, resp);

        let entries = deepl
            .retrieve_glossary_entries(&resp.glossary_id)
            .await
            .unwrap();
        assert_eq!(entries.len(), 2);
        let entries: HashMap<String, String> = HashMap::from_iter(entries);
        assert_eq!(entries["Hello"], "Guten Tag");
        assert_eq!(entries["Bye"], "Auf Wiedersehen");

        deepl.delete_glossary(resp.glossary_id).await.unwrap();
    }
}
