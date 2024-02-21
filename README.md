# DeepL Api

[<img alt="github" src="https://img.shields.io/badge/github-Avimitin/deepl--rs-7E9CD8?style=flat&labelColor=252535&logo=github" height="20">](https://github.com/Avimitin/deepl-rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/deepl.svg?style=flat&color=fd7726&labelColor=252535&logo=rust" height="20">](https://crates.io/crates/deepl)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/deepl?color=2b5a28&logo=rust&labelColor=252535" height="20">](https://docs.rs/deepl/latest/deepl/)

Typed HTTP wrapper for interacting with DeepL API. File upload/download is also implemented.

## Usage

```toml
[dependencies]
deepl = "0.5"
```

```rust
use deepl::{DeepLApi, Lang};

let api = DeepLApi::with("YOUR AUTH KEY").new();
let translated = api.translate_text("Hello World", Lang::ZH).await.unwrap();

let sentences = translated.translations;
assert_eq!(sentences[0].text, "你好，世界");
```

Read [examples](./examples) for more usage.

## Collaboration

If you find any bugs in this project or feel confused about any part of the code,
feel free to open new issue.

If you want to submit some code modification but don't know how to setup the
code environment, you can follow the
[Nix Installation](https://nixos.org/manual/nix/stable/installation/installing-binary.html#installing-a-binary-distribution)
and [enable flakes support](https://nixos.wiki/wiki/Flakes#Enable_flakes).
Then simply run `nix develop` in the project root, all the build dependencies will setup
for you.

## License

[MIT](./LICENSE)
