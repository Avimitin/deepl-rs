use super::{Pollable, Result};
use crate::{impl_requester, Formality, Lang};
use serde::{Deserialize, Serialize};
use std::{
    future::IntoFuture,
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

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

/// Response from api/v2/document/$ID
#[derive(Deserialize, Debug)]
pub struct DocumentStatusResp {
    /// A unique ID assigned to the uploaded document and the requested translation process.
    /// The same ID that was used when requesting the translation status.
    pub document_id: String,
    /// A short description of the state the document translation process is currently in.
    /// See [`DocumentTranslateStatus`] for more.
    pub status: DocumentTranslateStatus,
    /// Estimated number of seconds until the translation is done.
    /// This parameter is only included while status is "translating".
    pub seconds_remaining: Option<u64>,
    /// The number of characters billed to your account.
    pub billed_characters: Option<u64>,
    /// A short description of the error, if available. Note that the content is subject to change.
    /// This parameter may be included if an error occurred during translation.
    pub error_message: Option<String>,
}

/// Possible value of the document translate status
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentTranslateStatus {
    /// The translation job is waiting in line to be processed
    Queued,
    /// The translation is currently ongoing
    Translating,
    /// The translation is done and the translated document is ready for download
    Done,
    /// An irrecoverable error occurred while translating the document
    Error,
}

impl DocumentTranslateStatus {
    pub fn is_done(&self) -> bool {
        self == &Self::Done
    }
}

impl_requester! {
    UploadDocumentRequester {
        @required{
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

impl<'a> UploadDocumentRequester<'a> {
    fn to_multipart_form(&self) -> reqwest::multipart::Form {
        let Self {
            source_lang,
            target_lang,
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

        // SET formality
        if let Some(formal) = formality {
            form = form.text("formality", formal.to_string());
        }

        // SET glossary
        if let Some(id) = glossary_id {
            form = form.text("glossary_id", id.to_string());
        }

        form
    }

    fn send(&self) -> Pollable<'a, Result<UploadDocumentResp>> {
        let mut form = self.to_multipart_form();
        let client = self.client.clone();
        let filename = self.filename.clone();
        let file_path = self.file_path.clone();

        let fut = async move {
            // SET file && filename asynchronously
            let file = tokio::fs::read(&file_path).await.map_err(|err| {
                Error::ReadFileError(file_path.to_str().unwrap().to_string(), err)
            })?;

            let mut part = reqwest::multipart::Part::bytes(file);
            if let Some(filename) = filename {
                part = part.file_name(filename.to_string());
                form = form.text("filename", filename);
            } else {
                part = part.file_name(file_path.file_name().expect(
                    "No extension found for this file, and no filename given, cannot make request",
                ).to_str().expect("not a valid UTF-8 filepath!").to_string());
            }

            form = form.part("file", part);

            let res = client
                .post(client.get_endpoint("document"))
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
        };

        Box::pin(fut)
    }
}

impl<'a> IntoFuture for UploadDocumentRequester<'a> {
    type Output = Result<UploadDocumentResp>;
    type IntoFuture = Pollable<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        self.send()
    }
}

impl<'a> IntoFuture for &mut UploadDocumentRequester<'a> {
    type Output = Result<UploadDocumentResp>;
    type IntoFuture = Pollable<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        self.send()
    }
}

impl DeepLApi {
    /// Upload document to DeepL API server, return [`UploadDocumentResp`] for
    /// querying the translation status and to download the translated document once
    /// translation is complete.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepl::DeepLApi;
    ///
    /// let key = std::env::var("DEEPL_API_KEY").unwrap();
    /// let deepl = DeepLApi::with(&key).new();
    ///
    /// // Upload the file to DeepL
    /// let filepath = std::path::PathBuf::from("./hamlet.txt");
    /// let response = deepl.upload_document(&filepath, Lang::ZH)
    ///         .source_lang(Lang::EN)
    ///         .filename("Hamlet.txt".to_string())
    ///         .formality(Formality::Default)
    ///         .glossary_id("def3a26b-3e84-45b3-84ae-0c0aaf3525f7".to_string())
    ///         .await
    ///         .unwrap();
    /// ```
    ///
    /// Read the example `upload_document` in repository for detailed usage
    pub fn upload_document(
        &self,
        fp: impl Into<std::path::PathBuf>,
        target_lang: Lang,
    ) -> UploadDocumentRequester<'_> {
        UploadDocumentRequester::new(self, fp.into(), target_lang)
    }

    async fn open_file_to_write(p: &Path) -> Result<tokio::fs::File> {
        let open_result = tokio::fs::OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(p)
            .await;

        if let Ok(file) = open_result {
            return Ok(file);
        }

        let err = open_result.unwrap_err();
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(Error::WriteFileError(format!(
                "Fail to open file {p:?}: {err}"
            )));
        }

        tokio::fs::remove_file(p).await.map_err(|err| {
            Error::WriteFileError(format!(
                "There was already a file there and it is not deletable: {err}"
            ))
        })?;
        dbg!("Detect exist, removed");

        let open_result = tokio::fs::OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(p)
            .await;

        if let Err(err) = open_result {
            return Err(Error::WriteFileError(format!(
                "Fail to open file for download document, even after retry: {err}"
            )));
        }

        Ok(open_result.unwrap())
    }

    /// Check the status of document, returning [`DocumentStatusResp`] if success.
    pub async fn check_document_status(
        &self,
        ident: &UploadDocumentResp,
    ) -> Result<DocumentStatusResp> {
        let form = [("document_key", ident.document_key.as_str())];
        let url = self.get_endpoint(&format!("document/{}", ident.document_id));
        let res = self
            .post(url)
            .form(&form)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if !res.status().is_success() {
            return super::extract_deepl_error(res).await;
        }

        let status: DocumentStatusResp = res
            .json()
            .await
            .map_err(|err| Error::InvalidResponse(format!("response is not JSON: {err}")))?;

        Ok(status)
    }

    /// Download the possibly translated document. Downloaded document will store to the given
    /// `output` path.
    ///
    /// Return downloaded file's path if success
    pub async fn download_document<O: AsRef<Path>>(
        &self,
        ident: &UploadDocumentResp,
        output: O,
    ) -> Result<PathBuf> {
        let url = self.get_endpoint(&format!("document/{}/result", ident.document_id));
        let form = [("document_key", ident.document_key.as_str())];
        let res = self
            .post(url)
            .form(&form)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NonExistDocument);
        }

        if res.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            return Err(Error::TranslationNotDone);
        }

        if !res.status().is_success() {
            return super::extract_deepl_error(res).await;
        }

        let mut file = Self::open_file_to_write(output.as_ref()).await?;

        let mut stream = res.bytes_stream();

        #[inline]
        fn mapper<E: std::error::Error>(s: &'static str) -> Box<dyn FnOnce(E) -> Error> {
            Box::new(move |err: E| Error::WriteFileError(format!("{s}: {err}")))
        }

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(mapper("fail to download part of the document"))?;
            file.write_all(&chunk)
                .await
                .map_err(mapper("fail to write downloaded part into file"))?;
            file.sync_all()
                .await
                .map_err(mapper("fail to sync file content"))?;
        }

        Ok(output.as_ref().to_path_buf())
    }
}

#[tokio::test]
async fn test_upload_document() {
    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::with(&key).new();

    let raw_text = "Hello World";

    tokio::fs::write("./test.txt", &raw_text).await.unwrap();

    let test_file = PathBuf::from("./test.txt");
    let response = api.upload_document(&test_file, Lang::DE).await.unwrap();
    let mut status = api.check_document_status(&response).await.unwrap();

    // wait for translation
    loop {
        if status.status.is_done() {
            break;
        }
        if let Some(msg) = status.error_message {
            println!("{}", msg);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        status = api.check_document_status(&response).await.unwrap();
        dbg!(&status);
    }

    let path = api
        .download_document(&response, "test_translated.txt")
        .await
        .unwrap();

    let content = tokio::fs::read_to_string(path).await.unwrap();
    let expect = "Hallo Welt";
    assert_eq!(content, expect);
}

#[tokio::test]
async fn test_upload_docx() {
    use docx_rs::{read_docx, DocumentChild, Docx, Paragraph, ParagraphChild, Run, RunChild};

    let key = std::env::var("DEEPL_API_KEY").unwrap();
    let api = DeepLApi::with(&key).new();

    let test_file = PathBuf::from("./example.docx");
    let file = std::fs::File::create(&test_file).expect("fail to create test asserts");
    Docx::new()
        .add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text("To be, or not to be, that is the question")),
        )
        .build()
        .pack(file)
        .expect("fail to write test asserts");

    let response = api.upload_document(&test_file, Lang::DE).await.unwrap();
    let mut status = api.check_document_status(&response).await.unwrap();

    // wait for translation
    loop {
        if status.status.is_done() {
            break;
        }
        if let Some(msg) = status.error_message {
            println!("{}", msg);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        status = api.check_document_status(&response).await.unwrap();
        dbg!(&status);
    }

    let path = api
        .download_document(&response, "translated.docx")
        .await
        .unwrap();
    let get = tokio::fs::read(&path).await.unwrap();
    let doc = read_docx(&get).expect("can not open downloaded document");
    // collect all the text in this docx file
    let text = doc
        .document
        .children
        .iter()
        .filter_map(|child| {
            if let DocumentChild::Paragraph(paragraph) = child {
                let text = paragraph
                    .children
                    .iter()
                    .filter_map(|pchild| {
                        if let ParagraphChild::Run(run) = pchild {
                            let text = run
                                .children
                                .iter()
                                .filter_map(|rchild| {
                                    if let RunChild::Text(text) = rchild {
                                        Some(text.text.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<String>();

                            Some(text)
                        } else {
                            None
                        }
                    })
                    .collect::<String>();
                Some(text)
            } else {
                None
            }
        })
        .collect::<String>();

    assert_eq!(text, "Sein oder Nichtsein, das ist hier die Frage");
}
