# Changelog

## v0.7.3 - 2025-12-01

- Allow `String` and `Vec<String>` as translate API input

## v0.7.2 - 2025-11-22

- Fix en_US and en_GB description

## v0.7.1 - 2025-11-12

- Bump dependencies
```text
name         old_req  latest
====         =======  ======
thiserror    2.0.12   2.0.17
reqwest      0.12.22  0.12.14
serde        1.0.219  1.0.228
serde_json   1.0.140  1.0.145
tokio        1.46.1   1.48.0
```

## v0.7.0 - 2025-11-12

> Thanks GitHub@ValuedMammal for all the contributions.

- Add `GloassaryLanguage` enumeration and change `GloassaryLanguagePair` to use it. (API BREAKING)
- `glossary::EntriesFormat` now have `Display` implementation.
- Add `Error::Network` variant to hold client side network issue.
- Add multiple language to `Lang` variant.
    * Hebrew
    * Thai
    * Vietnamese
    * Spanish (Latin America)
- Fix internal clippy warning.

## v0.6.6 - 2025-07-17

- Add support for choosing different model
- Bump dependencies

```text
name         old req compatible latest  new req
====         ======= ========== ======  =======
thiserror    2.0.3   2.0.12     2.0.12  2.0.12
reqwest      0.12.9  0.12.22    0.12.22 0.12.22
serde        1.0.215 1.0.219    1.0.219 1.0.219
serde_json   1.0.133 1.0.140    1.0.140 1.0.140
tokio        1.41.1  1.46.1     1.46.1  1.46.1
tokio-stream 0.1.16  0.1.17     0.1.17  0.1.17
```

## v0.6.5 - 2024-12-03

- Refactor translate API with JSON parser
- Bump all dependencies
```text
name         old req latest
====         ======= ======
thiserror    1.0.63  2.0.3
reqwest      0.12.7  0.12.9
serde        1.0.208 1.0.215
serde_json   1.0.125 1.0.133
tokio        1.39.3  1.41.1
tokio-stream 0.1.15  0.1.16
```

## v0.6.4 - 2024-08-21
- Add derive of `hash` for `Lang`
- Add `ZH-HANS` and `ZH-HANT`
- Bump dependencies:
    * thiserror: `1.0.35` -> `1.0.63`
    * reqwest: `0.12.4` -> `0.12.7`
    * serde: `1.0.144` -> `1.0.208`
    * tokio: `1.21.1` -> `1.39.3`
    * tokio-stream: `0.1.11` -> `0.1.15`
    * paste: `1.0.11` -> `1.0.15`
    * typed-builder: `0.18` -> `0.19`

## v0.6.3 - 2024-04-28

- Add FromStr and Display trait implementation for Lang

## v0.6.2 - 2024-03-19

- Add support for AR (Arabic)
- Support automatically switch API backend

## v0.6.1 - 2024-03-01

- Wrap panic into Result

## v0.6.0 - 2024-02-21

- Fix incorrect implmentation of glossary API

## v0.5.1 - 2024-02-21

- Add `context` field into translate API
- make all field in glossary API as public

## v0.5.0 - 2024-02-21

- Add type constraint for glossary APIs

## v0.4.6 - 2024-02-14

- Implement all the glossaries related API

## v0.4.5 - 2024-01-26
- Re-export the `LangConvertError` struct

## v0.4.4 - 2023-11-20

- Add new `languages` endpoint
- Add `KO` and `NB` language variant

## [v0.4.3] - 2023-09-11

- Improve code document

## [v0.4.2] - 2023-06-23

- Include formality in impl_requester

## [v0.4.1] - 2023-02-23

- Add `Clone` derive for `Lang`

## [v0.4.0] - 2023-01-26

### Changed

- (**BREAKING**) Implement auto send for all endpoint
- (**BREAKING**) `DeepLApi` implementation is now separated to multiple endpoint file
- (**BREAKING**) `DeepLApiResponse` is now renamed to `TranslateTextResp`
- (**BREAKING**) `DeepLApi` is now init by `::with()` function and build by `.new()` function
- Using `docx-rs` to parse document content for testing

## [v0.3.0] - 2023-01-10

### Changed

- (**BREAKING**) `Lang::from` is now replaced with `Lang::try_from`

## [v0.2.0] - 2023-01-08

### Added

- Full API options for endpoint `translate`
- New builder for DeepLApi
- Complete some missing document

### Changed

- Correct all the typo
- `reqwest` crate is re-exported
- (**BREAKING**) `translate` function only accept `TranslateTextProp` now

## [v0.1.6] - 2022-12-02

### Fixed

- Fix document download issue

### Changed

- Use `AsRef<Path>` as `UploadDocumentProp::file_path` type
- Use only `output` parameter for function `download_document`

## [v0.1.3] - 2022-11-29

### Added

- New upload document API
- More language variants implemented
- Allow user using Pro version API

## [v0.1.2] - 2022-09-20

### Added

- New API `get_usage()` to get current API usage

### Changed

- Replace Anyhow::Result with custom Error
