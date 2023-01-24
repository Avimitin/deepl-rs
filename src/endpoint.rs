use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};
use thiserror::Error;

pub mod document;
pub mod translate;

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

type Result<T, E = Error> = std::result::Result<T, E>;

type Pollable<'poll, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'poll>>;

pub trait ToPollable<T> {
    fn to_pollable(&mut self) -> Pollable<T>;
}

#[macro_export]
macro_rules! impl_requester {
    (
        $(#[$docs:meta])*
        $name:ident {
            @must{
                $($must_field:ident: $must_type:ty,)+
            };
            @optional{
                $($opt_field:ident: $opt_type:ty,)+
            };
        } -> $fut_ret:ty;
    ) => {
        use paste::paste;
        use crate::{DeepLApi, Error};

        paste! {
            $(#[$docs])*
            pub struct [<$name Requester>]<'a> {
                client: &'a DeepLApi,

                $($must_field: $must_type,)+
                $($opt_field: Option<$opt_type>,)+
            }

            impl<'a> [<$name Requester>]<'a> {
                pub fn new(client: &'a DeepLApi, $($must_field: $must_type,)+) -> Self {
                    Self {
                        client,
                        $($must_field,)+
                        $($opt_field: None,)+
                    }
                }

                $(
                    pub fn $opt_field(&mut self, $opt_field: $opt_type) -> &mut Self {
                        self.$opt_field = Some($opt_field);
                        self
                    }
                )+
            }

            impl<'a> std::future::Future for [<$name Requester>]<'a> {
                type Output = $fut_ret;

                fn poll(
                    mut self: std::pin::Pin<&mut Self>,
                    cx: &mut std::task::Context<'_>,
                ) -> std::task::Poll<Self::Output> {
                    let mut fut = self.to_pollable();
                    fut.as_mut().poll(cx)
                }
            }
        }
    };
}

#[derive(Serialize)]
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

impl ToString for Formality {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

// detail message of the API error
#[derive(Deserialize)]
struct DeepLErrorResp {
    message: String,
}

async fn extract_deepl_error(res: reqwest::Response) -> Result<DeepLErrorResp> {
    let resp = res
        .json::<DeepLErrorResp>()
        .await
        .map_err(|err| Error::InvalidResponse(format!("invalid error response: {err}")))?;
    Err(Error::RequestFail(resp.message))
}
