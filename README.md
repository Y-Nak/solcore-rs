# solcore-rs

Experimental Core Solidity Compiler
This is a prototype implementation of Solidity's new type system. It is functional and can produce executable EVM code.

> [!WARNING]
> This project is a work in progress and is not ready for production use.

Try it in the [online playground](https://playground.solcore.soliditylang.org/).

## Language and standard library

The Rust compiler and its regression tests define the supported Solcore
language. Sources use the canonical `.sol` syntax. The eight library sources
in [`std/`](std/) are maintained with this compiler; see the standard library's
[README](std/README.md) for its maintenance and verification policy.

The compiler reports structured diagnostics and validates ABI support before
backend execution. The 499-source parser corpus is a fixed historical
regression set, as documented in its
[README](crates/parser/tests/fixtures/corpus/README.md).

## Build and test

[Rust](https://www.rust-lang.org/tools/install) is required. The checked-in
`rust-toolchain.toml` selects Rust 1.97.0 automatically when using rustup.

```sh
cargo build --workspace --locked
cargo test --workspace --locked
```

## Run the compiler

From the repository root, compile an example and print the generated Yul:

```sh
cargo run -p solcore-driver -- tests/e2e/00answer/main.sol --emit-yul
```

Replace the example path with your own `.sol` file. For all available options:

```sh
cargo run -p solcore-driver -- --help
```

## Formatting Solcore

The standalone `solcore-fmt` command formats UTF-8 Solcore source files:

```sh
# Print formatted source without changing the file.
cargo run -p solcore-fmt -- main.sol

# Read stdin and print the result.
cargo run -p solcore-fmt -- < main.sol

# Format one or more files in place.
cargo run -p solcore-fmt -- --write main.sol library.sol

# Check formatting in CI without changing files.
cargo run -p solcore-fmt -- --check main.sol

# Install the standalone command locally.
cargo install --path crates/fmt
```

Use `solcore-fmt --help` for all options. Indentation defaults to four spaces;
`--indent-width N` changes it (1–16 spaces). A `-` input reads stdin. Multiple
inputs are printed in argument order unless `--write` or `--check` is selected.
Inputs are individual files; directories are not searched recursively. `--check` exits
with status 1 when formatting differs; invalid syntax, arguments, or I/O errors
exit with status 2.

## License

Licensed under the [Apache License 2.0](LICENSE).
