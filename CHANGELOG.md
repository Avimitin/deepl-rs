# Changelog

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
