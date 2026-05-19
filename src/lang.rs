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
            pub enum Lang {
                $(
                    #[doc = $desc]
                    #[serde(rename = $code)]
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
    ("AF",    "Afrikaans");
    ("AN",    "Aragonese");
    ("AR",    "Arabic");
    ("AS",    "Assamese");
    ("AY",    "Aymara");
    ("AZ",    "Azerbaijani");
    ("BA",    "Bashkir");
    ("BE",    "Belarusian");
    ("BG",    "Bulgarian");
    ("BN",    "Bengali");
    ("BR",    "Breton");
    ("BS",    "Bosnian");
    ("CA",    "Catalan");
    ("CS",    "Czech");
    ("CY",    "Welsh");
    ("DA",    "Danish");
    ("DE",    "German");
    ("DE-DE", "German (Germany)");
    ("EL",    "Greek");
    ("EN",    "English (Unspecified variant)");
    ("EN-GB", "English (British)");
    ("EN-US", "English (American)");
    ("EO",    "Esperanto");
    ("ES",    "Spanish");
    ("ES-419", "Spanish (Latin America)");
    ("ET",    "Estonian");
    ("EU",    "Basque");
    ("FA",    "Persian");
    ("FI",    "Finnish");
    ("FR",    "French");
    ("FR-FR", "French (France)");
    ("GA",    "Irish");
    ("GL",    "Galician");
    ("GN",    "Guarani");
    ("GU",    "Gujarati");
    ("HA",    "Hausa");
    ("HE",    "Hebrew");
    ("HI",    "Hindi");
    ("HR",    "Croatian");
    ("HT",    "Haitian Creole");
    ("HU",    "Hungarian");
    ("HY",    "Armenian");
    ("ID",    "Indonesian");
    ("IG",    "Igbo");
    ("IS",    "Icelandic");
    ("IT",    "Italian");
    ("JA",    "Japanese");
    ("JV",    "Javanese");
    ("KA",    "Georgian");
    ("KK",    "Kazakh");
    ("KO",    "Korean");
    ("KY",    "Kyrgyz");
    ("LA",    "Latin");
    ("LB",    "Luxembourgish");
    ("LN",    "Lingala");
    ("LT",    "Lithuanian");
    ("LV",    "Latvian");
    ("MG",    "Malagasy");
    ("MI",    "Maori");
    ("MK",    "Macedonian");
    ("ML",    "Malayalam");
    ("MN",    "Mongolian");
    ("MR",    "Marathi");
    ("MS",    "Malay");
    ("MT",    "Maltese");
    ("MY",    "Burmese");
    ("NB",    "Norwegian (bokmål)");
    ("NE",    "Nepali");
    ("NL",    "Dutch");
    ("OC",    "Occitan");
    ("OM",    "Oromo");
    ("PA",    "Punjabi");
    ("PL",    "Polish");
    ("PS",    "Pashto");
    ("PT",    "Portuguese (all Portuguese varieties mixed)");
    ("PT-BR", "Portuguese (Brazilian)");
    ("PT-PT", "Portuguese (European)");
    ("QU",    "Quechua");
    ("RO",    "Romanian");
    ("RU",    "Russian");
    ("SA",    "Sanskrit");
    ("SK",    "Slovak");
    ("SL",    "Slovenian");
    ("SQ",    "Albanian");
    ("SR",    "Serbian");
    ("ST",    "Sesotho");
    ("SU",    "Sundanese");
    ("SV",    "Swedish");
    ("SW",    "Swahili");
    ("TA",    "Tamil");
    ("TE",    "Telugu");
    ("TG",    "Tajik");
    ("TH",    "Thai");
    ("TK",    "Turkmen");
    ("TL",    "Tagalog");
    ("TN",    "Tswana");
    ("TR",    "Turkish");
    ("TS",    "Tsonga");
    ("TT",    "Tatar");
    ("UK",    "Ukrainian");
    ("UR",    "Urdu");
    ("UZ",    "Uzbek");
    ("VI",    "Vietnamese");
    ("WO",    "Wolof");
    ("XH",    "Xhosa");
    ("YI",    "Yiddish");
    ("ZH",    "Chinese");
    ("ZH-HANS", "Chinese (simplified)");
    ("ZH-HANT", "Chinese (traditional)");
    ("ZU",    "Zulu");
}

impl<'de> Deserialize<'de> for Lang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lang = String::deserialize(deserializer)?.to_uppercase();

        let lang = Lang::try_from(&lang).map_err(|_| {
            serde::de::Error::custom(
                format!("invalid language code {lang}. This is an internal issue with the lib, please open an issue!")
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
