# DeepL Api

[<img alt="github" src="https://img.shields.io/badge/github-Avimitin/deepl--rs-7E9CD8?style=flat&labelColor=252535&logo=github" height="20">](https://github.com/Avimitin/deepl-rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/deepl.svg?style=flat&color=fd7726&labelColor=252535&logo=rust" height="20">](https://crates.io/crates/deepl)

Interact with DeepL API with typed wrapper.

## Usage

```toml
[dependencies]
deepl = "0.1.0"
```

```rust
use deepl::{DeepLApi, Lang};

let api = DeepLApi::new("YOUR AUTH KEY");

// Source lang is optional
let translated = api.translate("Hello World", Some(Lang::EN), Lang::ZH).await.unwrap();

assert!(!translated.translations.is_empty());

let sentences = translated.translations;
assert_eq!(sentences[0].text, "你好，世界");
```

## License

[MIT](./LICENSE)
