use super::{Formality, Pollable, Result, ToPollable};
use crate::{impl_requester, Lang};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Response from api/v2/document
#[derive(Serialize, Deserialize)]
pub struct UploadDocumentResp {
    /// A unique ID assigned to the uploaded document and the translation process.
    /// Must be used when referring to this particular document in subsequent API requests.
    pub document_id: String,
    /// A unique key that is used to encrypt the uploaded document as well as the resulting
    /// translation on the server side. Must be provided with every subsequent API request
    /// regarding this particular document.
    pub document_key: String,
}

impl_requester! {
    UploadDocument {
        @must{
            file_path: PathBuf,
            target_lang: Lang,
        };
        @optional{
            source_lang: Lang,
            filename: String,
            formality: Formality,
            glossary_id: String,
        };
    } -> Result<UploadDocumentResp, Error>;
}

impl<'a> ToPollable<Result<UploadDocumentResp>> for UploadDocumentRequester<'a> {
    fn to_pollable(&mut self) -> Pollable<Result<UploadDocumentResp>> {
        Box::pin(self.send())
    }
}

impl<'a> UploadDocumentRequester<'a> {
    async fn to_multipart_form(&self) -> Result<reqwest::multipart::Form, crate::Error> {
        let Self {
            source_lang,
            target_lang,
            file_path,
            filename,
            formality,
            glossary_id,
            ..
        } = self;

        let mut form = reqwest::multipart::Form::new();

        // SET source_lang
        if let Some(lang) = source_lang {
            form = form.text("source_lang", lang.to_string());
        }

        // SET target_lang
        form = form.text("target_lang", target_lang.to_string());

        // SET file && filename
        let file = tokio::fs::read(&file_path)
            .await
            .map_err(|err| Error::ReadFileError(file_path.to_str().unwrap().to_string(), err))?;

        let mut part = reqwest::multipart::Part::bytes(file);
        if let Some(filename) = filename {
            part = part.file_name(filename.to_string());
            form = form.text("filename", filename.to_string());
        } else {
            part = part.file_name(file_path.file_name().expect(
                "No extension found for this file, and no filename given, cannot make request",
            ).to_str().expect("no a valid UTF-8 filepath!").to_string());
        }

        form = form.part("file", part);

        // SET formality
        if let Some(formal) = formality {
            form = form.text("formality", formal.to_string());
        }

        // SET glossary
        if let Some(id) = glossary_id {
            form = form.text("glossary_id", id.to_string());
        }

        Ok(form)
    }

    async fn send(&self) -> Result<UploadDocumentResp> {
        let form = self.to_multipart_form().await?;
        let res = self
            .client
            .post(self.client.endpoint.join("document").unwrap())
            .multipart(form)
            .send()
            .await
            .map_err(|err| Error::RequestFail(format!("fail to upload file: {err}")))?;

        if !res.status().is_success() {
            return super::extract_deepl_error(res).await;
        }

        let res: UploadDocumentResp = res.json().await.map_err(|err| {
            Error::InvalidResponse(format!("fail to decode response body: {err}"))
        })?;

        Ok(res)
    }
}
