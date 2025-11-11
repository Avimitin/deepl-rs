use serde::Serialize;
use std::{future::Future, pin::Pin};
use thiserror::Error;

pub mod document;
pub mod glossary;
pub mod languages;
pub mod translate;
pub mod usage;

/// Representing error during interaction with DeepL
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("request fail: {0}")]
    RequestFail(String),

    #[error("fail to read file {0}: {1}")]
    ReadFileError(String, tokio::io::Error),

    #[error(
        "trying to download a document using a non-existing document ID or the wrong document key"
    )]
    NonExistDocument,

    #[error("tries to download a translated document that is currently being processed and is not yet ready for download")]
    TranslationNotDone,

    #[error("fail to write file: {0}")]
    WriteFileError(String),
}

const REPO_URL: &str = "https://github.com/Avimitin/deepl-rs";

/// Alias Result<T, E> to Result<T, [`Error`]>
type Result<T, E = Error> = std::result::Result<T, E>;

/// Pollable alias to a Pin<Box<dyn Future<...>>>. A convenient type for impl
/// [`IntoFuture`](std::future::IntoFuture) trait
type Pollable<'poll, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'poll>>;

/// A self implemented Type Builder
#[macro_export]
macro_rules! impl_requester {
    (
        $name:ident {
            @required{
                $($must_field:ident: $must_type:ty,)+
            };
            @optional{
                $($opt_field:ident: $opt_type:ty,)*
            };
        } -> $fut_ret:ty;
    ) => {
        use paste::paste;
        use $crate::{DeepLApi, Error};

        paste! {
            #[doc = "Builder type for `" $name "`"]
            #[derive(Debug, serde::Serialize)]
            pub struct $name<'a> {
                #[serde(skip)]
                client: &'a DeepLApi,

                $($must_field: $must_type,)+
                $($opt_field: Option<$opt_type>,)*
            }

            impl<'a> $name<'a> {
                pub fn new(client: &'a DeepLApi, $($must_field: $must_type,)+) -> Self {
                    Self {
                        client,
                        $($must_field,)+
                        $($opt_field: None,)*
                    }
                }

                $(
                    #[doc = "Setter for `" $opt_field "`"]
                    pub fn $opt_field(&mut self, $opt_field: $opt_type) -> &mut Self {
                        self.$opt_field = Some($opt_field);
                        self
                    }
                )*
            }
        }
    };
}

/// Formality preference for translation
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Formality {
    Default,
    More,
    Less,
    PreferMore,
    PreferLess,
}

impl AsRef<str> for Formality {
    fn as_ref(&self) -> &str {
        match self {
            Self::Default => "default",
            Self::More => "more",
            Self::Less => "less",
            Self::PreferMore => "prefer_more",
            Self::PreferLess => "prefer_less",
        }
    }
}

impl std::fmt::Display for Formality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// Turn DeepL API error message into [`Error`]
async fn extract_deepl_error<T>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    match resp.text().await.ok() {
        Some(message) => Err(Error::RequestFail(format!("{status} {message}"))),
        None => Err(Error::RequestFail(format!("{status}"))),
    }
}
