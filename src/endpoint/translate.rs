use crate::{DeepLApi, DeepLApiResponse, Lang, PreserveFormatting, SplitSentences, TagHandling};
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
                    todo!()
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
            ignore_tags: Vec<String>,
        };
    } -> String;
}
