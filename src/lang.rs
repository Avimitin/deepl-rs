use std::{fmt::Display, str::FromStr};

use paste::paste;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LangConvertError {
    #[error("invalid language code {0}")]
    InvalidLang(String),
}

type Result<T, E = LangConvertError> = core::result::Result<T, E>;

macro_rules! generate_langs {
    (
        $(
            ($code:literal, $desc:literal);
        )+
    ) => {
        paste! {
            /// Languages
            #[allow(non_camel_case_types)]
            #[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize)]
            #[serde(rename_all = "SCREAMING-KEBAB-CASE")]
            pub enum Lang {
                $(
                    #[doc = $desc]
                    [<$code>],
                )+
            }

            impl Lang {
                /// Return full language name for the code
                pub fn description(&self) -> String {
                    match self {
                        $(
                            Self::[<$code>] => $desc.to_string(),
                        )+
                    }
                }
            }

            impl TryFrom<&str> for Lang {
                type Error = LangConvertError;

                /// Convert literal to enum `Lang`
                ///
                /// # Error
                ///
                /// Return `Error::InvalidLang` when given language code is not in the support list.
                fn try_from(value: &str) -> Result<Self, Self::Error> {
                    let lang = match value {
                        $(
                            $code => Self::[<$code>],
                        )+
                        _ => return Err(LangConvertError::InvalidLang(value.to_string())),
                    };

                    Ok(lang)
                }
            }

            impl TryFrom<&String> for Lang {
                type Error = LangConvertError;

                /// Convert ref String to enum `Lang`
                ///
                /// # Error
                ///
                /// Return `Error::InvalidLang` when given language code is not in the support list.
                fn try_from(value: &String) -> Result<Self, Self::Error> {
                    let lang = match value.as_ref() {
                        $(
                            $code => Self::[<$code>],
                        )+
                        _ => return Err(LangConvertError::InvalidLang(value.to_string())),
                    };

                    Ok(lang)
                }
            }

            impl AsRef<str> for Lang {
                fn as_ref(&self) -> &'static str {
                    match self {
                        $(
                            Self::[<$code>] => $code,
                        )+
                    }
                }
            }
        }
    };
}

generate_langs! {
    ("AR",    "Arabic");
    ("BG",    "Bulgarian");
    ("CS",    "Czech");
    ("DA",    "Danish");
    ("DE",    "German");
    ("EL",    "Greek");
    ("EN",    "English (Unspecified variant)");
    ("EN-GB", "English (American)");
    ("EN-US", "English (British)");
    ("ES",    "Spanish");
    ("ET",    "Estonian");
    ("FI",    "Finnish");
    ("FR",    "French");
    ("HU",    "Hungarian");
    ("ID",    "Indonesian");
    ("IT",    "Italian");
    ("JA",    "Japanese");
    ("KO",    "Korean");
    ("LT",    "Lithuanian");
    ("LV",    "Latvian");
    ("NB",    "Norwegian");
    ("NL",    "Dutch");
    ("PL",    "Polish");
    ("PT",    "Portuguese (all Portuguese varieties mixed)");
    ("PT-BR", "Portuguese (Brazilian)");
    ("PT-PT", "Portuguese (All Portuguese varieties excluding Brazilian)");
    ("RO",    "Romanian");
    ("RU",    "Russian");
    ("SK",    "Slovak");
    ("SL",    "Slovenian");
    ("SV",    "Swedish");
    ("TR",    "Turkish");
    ("UK",    "Ukrainian");
    ("ZH",    "Chinese");
}

impl<'de> Deserialize<'de> for Lang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lang = String::deserialize(deserializer)?.to_uppercase();

        let lang = Lang::try_from(&lang).map_err(|_| {
            serde::de::Error::custom(
                format!("invalid language code {lang}. This is an internal issue with the lib, please open issue")
            )
        })?;

        Ok(lang)
    }
}

impl FromStr for Lang {
    type Err = LangConvertError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Lang::try_from(s)
    }
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
