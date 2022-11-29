use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LangConvertError {
    #[error("invalid language code {0}")]
    InvalidLang(String),
}

type Result<T, E = LangConvertError> = core::result::Result<T, E>;

/// Available language code for source and target text
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Lang {
    BG,
    CS,
    DA,
    DE,
    EL,
    EN,
    EN_GB,
    EN_US,
    ES,
    ET,
    FI,
    FR,
    HU,
    ID,
    IT,
    JA,
    LT,
    LV,
    NL,
    PL,
    PT,
    PT_BR,
    PT_PT,
    RO,
    RU,
    SK,
    SL,
    SV,
    TR,
    UK,
    ZH,
}

impl Lang {
    /// Convert literal to enum `Lang`
    ///
    /// # Error
    ///
    /// Return `Error::InvalidLang` when given language code is not in the support list.
    pub fn from(s: &str) -> Result<Self> {
        let lang = match s {
            "BG" => Self::BG,
            "CS" => Self::CS,
            "DA" => Self::DA,
            "DE" => Self::DE,
            "EL" => Self::EL,
            "EN" => Self::EN,
            "EN-GB" => Self::EN_GB,
            "EN-US" => Self::EN_US,
            "ES" => Self::ES,
            "ET" => Self::ET,
            "FI" => Self::FI,
            "FR" => Self::FR,
            "HU" => Self::HU,
            "ID" => Self::ID,
            "IT" => Self::IT,
            "JA" => Self::JA,
            "LT" => Self::LT,
            "LV" => Self::LV,
            "NL" => Self::NL,
            "PL" => Self::PL,
            "PT" => Self::PT,
            "PT-BR" => Self::PT_BR,
            "PT-PT" => Self::PT_PT,
            "RO" => Self::RO,
            "RU" => Self::RU,
            "SK" => Self::SK,
            "SL" => Self::SL,
            "SV" => Self::SV,
            "TR" => Self::TR,
            "UK" => Self::UK,
            "ZH" => Self::ZH,
            _ => return Err(LangConvertError::InvalidLang(s.to_string())),
        };

        Ok(lang)
    }

    /// Return full language name for the code
    pub fn description(&self) -> String {
        match self {
            Self::BG => "Bulgarian".to_string(),
            Self::CS => "Czech".to_string(),
            Self::DA => "Danish".to_string(),
            Self::DE => "German".to_string(),
            Self::EL => "Greek".to_string(),
            Self::EN => "English (Unspecified variant)".to_string(),
            Self::EN_US => "English (American)".to_string(),
            Self::EN_GB => "English (British)".to_string(),
            Self::ES => "Spanish".to_string(),
            Self::ET => "Estonian".to_string(),
            Self::FI => "Finnish".to_string(),
            Self::FR => "French".to_string(),
            Self::HU => "Hungarian".to_string(),
            Self::ID => "Indonesian".to_string(),
            Self::IT => "Italian".to_string(),
            Self::JA => "Japanese".to_string(),
            Self::LT => "Lithuanian".to_string(),
            Self::LV => "Latvian".to_string(),
            Self::NL => "Dutch".to_string(),
            Self::PL => "Polish".to_string(),
            Self::PT => "Portuguese (all Portuguese varieties mixed)".to_string(),
            Self::PT_BR => "Portuguese (Brazilian)".to_string(),
            Self::PT_PT => "Portuguese (All Portuguese varieties excluding Brazilian)".to_string(),
            Self::RO => "Romanian".to_string(),
            Self::RU => "Russian".to_string(),
            Self::SK => "Slovak".to_string(),
            Self::SL => "Slovenian".to_string(),
            Self::SV => "Swedish".to_string(),
            Self::TR => "Turkish".to_string(),
            Self::UK => "Ukrainian".to_string(),
            Self::ZH => "Chinese".to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for Lang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lang = String::deserialize(deserializer)?;

        let lang = Lang::from(&lang).map_err(|_| {
            serde::de::Error::custom(
                format!("invalid language code {lang}. This is an internal issue with the lib, please open issue")
            )
        })?;

        Ok(lang)
    }
}

impl AsRef<str> for Lang {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::BG => "BG",
            Self::CS => "CS",
            Self::DA => "DA",
            Self::DE => "DE",
            Self::EL => "EL",
            Self::EN => "EN",
            Self::EN_US => "EN-US",
            Self::EN_GB => "EN-GB",
            Self::ES => "ES",
            Self::ET => "ET",
            Self::FI => "FI",
            Self::FR => "FR",
            Self::HU => "HU",
            Self::ID => "ID",
            Self::IT => "IT",
            Self::JA => "JA",
            Self::LT => "LT",
            Self::LV => "LV",
            Self::NL => "NL",
            Self::PL => "PL",
            Self::PT => "PT",
            Self::PT_BR => "PT-BR",
            Self::PT_PT => "PT-PT",
            Self::RO => "RO",
            Self::RU => "RU",
            Self::SK => "SK",
            Self::SL => "SL",
            Self::SV => "SV",
            Self::TR => "TR",
            Self::UK => "UK",
            Self::ZH => "ZH",
        }
    }
}

impl ToString for Lang {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}
