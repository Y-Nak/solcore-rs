# solcore-rs

`solcore-rs` is a Rust implementation of [Solcore](https://github.com/argotorg/solcore).

> [!WARNING]
> This project is a work in progress and is not ready for production use.

Try it in the [online playground](https://playground.solcore.soliditylang.org/).

## Language and compatibility targets

The source-language and standard-library target is the Haskell implementation
at [`argotorg/solcore@a7b7afb0`](https://github.com/argotorg/solcore/tree/a7b7afb0638e50895008d778a167afd0e50a2c86).
Both implementations now use the canonical `.sol`
[source syntax](https://github.com/argotorg/solcore/blob/a7b7afb0638e50895008d778a167afd0e50a2c86/doc/src/sail/syntax.md).
The eight library sources in `std/` match that revision byte-for-byte.

See [`SEMANTIC_DIFFERENCES.md`](SEMANTIC_DIFFERENCES.md) for intentional
corrections to upstream defects, ABI limitations, and validation scope. Rust
retains structured diagnostics and safety checks; compatibility does not imply
identical compiler output or production readiness. The 499-source parser
corpus remains a historical `2f372bde` regression set, as documented in its
[README](crates/parser/tests/fixtures/corpus/README.md).

## Build and test

[Rust](https://www.rust-lang.org/tools/install) is required. The checked-in
`rust-toolchain.toml` selects Rust 1.97.0 automatically when using rustup.

```sh
cargo build --workspace --locked
cargo test --workspace --locked
```

## License

Licensed under the [Apache License 2.0](LICENSE).
