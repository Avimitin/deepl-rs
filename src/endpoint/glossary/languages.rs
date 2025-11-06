use paste::paste;

macro_rules! impl_glossary_languages {
    ( $($lang:literal, $name:literal $(,)? )+ ) => {
        paste! {
            /// Glossary languages.
            #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "lowercase")]
            pub enum GlossaryLanguage {
                $(
                    #[doc = $name]
                    [<$lang:camel>],
                )+
            }

            use crate::lang::LangConvertError;

            impl core::str::FromStr for GlossaryLanguage {
                type Err = LangConvertError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $(
                            $lang => Ok(Self::[<$lang:camel>]),
                        )+
                        _ => Err(LangConvertError::InvalidLang(s.to_string())),
                    }
                }
            }

            impl AsRef<str> for GlossaryLanguage {
                fn as_ref(&self) -> &str {
                    match self {
                        $(
                            Self::[<$lang:camel>] => $lang,
                        )+
                    }
                }
            }

            impl core::fmt::Display for GlossaryLanguage {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{}", self.as_ref())
                }
            }
        }
    }
}

#[rustfmt::skip]
impl_glossary_languages!(
    "ar", "Arabic",
    "bg", "Bulgarian",
    "cs", "Czech",
    "da", "Danish",
    "de", "German",
    "el", "Greek",
    "en", "English",
    "es", "Spanish",
    "et", "Estonian",
    "fi", "Finish",
    "fr", "French",
    "he", "Hebrew",
    "hu", "Hungarian",
    "id", "Indonesian",
    "it", "Italian",
    "ja", "Japanese",
    "ko", "Korean",
    "lt", "Lithuanian",
    "lv", "Latvian",
    "nb", "Norwegian",
    "nl", "Dutch",
    "pl", "Polish",
    "pt", "Portuguese",
    "ro", "Romanian",
    "ru", "Russian",
    "sk", "Slovak",
    "sl", "Slovenian",
    "sv", "Swedish",
    "th", "Thai",
    "tr", "Turkish",
    "uk", "Ukranian",
    "vi", "Vietnamese",
    "zh", "Chinese",
);
