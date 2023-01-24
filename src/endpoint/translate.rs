use std::{collections::HashMap, future::Future, pin::Pin};

use crate::{
    DeepLApi, DeepLApiResponse, Error, Lang, PreserveFormatting, SplitSentences, TagHandling,
};
use paste::paste;

macro_rules! impl_requester {
    (
        $name:ident {
            @must{
                $($must_field:ident: $must_type:ty,)+
            };
            @optional{
                $($opt_field:ident: $opt_type:ty,)+
            };
        } -> $fut_ret:ty;
    ) => {
        paste! {
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
                    self: std::pin::Pin<&mut Self>,
                    cx: &mut std::task::Context<'_>,
                ) -> std::task::Poll<Self::Output> {
                    let mut fut = self.to_pollable();
                    fut.as_mut().poll(cx)
                }
            }
        }
    };
}

impl_requester! {
    Translate {
        @must{
            text: String,
            target_lang: Lang,
        };
        @optional{
            source_lang: Lang,
            split_sentences: SplitSentences,
            preserve_formatting: PreserveFormatting,
            glossary_id: String,
            tag_handling: TagHandling,
            non_splitting_tags: Vec<String>,
            splitting_tags: Vec<String>,
            ignore_tags: Vec<String>,
        };
    } -> Result<DeepLApiResponse, Error>;
}

type Pollable<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;

pub trait ToPollable<T> {
    fn to_pollable(self) -> Pollable<T>;
}

impl<'a> ToPollable<Result<DeepLApiResponse, Error>> for TranslateRequester<'a> {
    fn to_pollable(self) -> Pollable<Result<DeepLApiResponse, Error>> {
        Box::pin(self.send())
    }
}

impl<'a> TranslateRequester<'a> {
    pub async fn send(&self) -> Result<DeepLApiResponse, Error> {
        let mut param = HashMap::new();
        param.insert("text", self.text.as_str());

        if let Some(ref la) = self.source_lang {
            param.insert("source_lang", la.as_ref());
        }
        param.insert("target_lang", self.target_lang.as_ref());
        if let Some(ref ss) = self.split_sentences {
            param.insert("split_sentences", ss.as_ref());
        }
        if let Some(ref pf) = self.preserve_formatting {
            param.insert("preserve_formatting", pf.as_ref());
        }
        if let Some(ref id) = self.glossary_id {
            param.insert("glossary_id", id);
        }
        if let Some(ref th) = self.tag_handling {
            param.insert("tag_handling", th.as_ref());
        }

        let ns_tags: String;
        if let Some(tags) = &self.non_splitting_tags {
            if !tags.is_empty() {
                ns_tags = tags.join(",");
                param.insert("non_splitting_tags", &ns_tags);
            }
        }

        let sp_tags: String;
        if let Some(tags) = &self.splitting_tags {
            if !tags.is_empty() {
                sp_tags = tags.join(",");
                param.insert("splitting_tags", &sp_tags);
            }
        }

        let ig_tags: String;
        if let Some(tags) = &self.ignore_tags {
            if !tags.is_empty() {
                ig_tags = tags.join(",");
                param.insert("ignore_tags", &ig_tags);
            }
        }

        let response = self
            .client
            .post(self.client.endpoint.join("translate").unwrap())
            .form(&param)
            .send()
            .await
            .map_err(|err| Error::RequestFail(err.to_string()))?;

        if !response.status().is_success() {
            return {
                let resp = response
                    .json::<crate::DeeplErrorResp>()
                    .await
                    .map_err(|err| {
                        Error::InvalidResponse(format!("invalid error response: {err}"))
                    })?;
                Err(Error::RequestFail(resp.message))
            };
        }

        let response: DeepLApiResponse = response.json().await.map_err(|err| {
            Error::InvalidResponse(format!("convert json bytes to Rust type: {err}"))
        })?;

        Ok(response)
    }
}
