# Solcore standard library

The eight `.sol` files in this directory are the standard library for the Rust
compiler. Format them with `solcore-fmt`. The copies under
`crates/parser/tests/fixtures/corpus/ok/std/` are parser fixtures and must remain
byte-for-byte identical to the files here.

## Maintenance policy

Maintain the library alongside the Rust compiler. Update and format each
changed source and its parser-corpus copy together. Cover language and ABI
changes with focused frontend, specialization, and backend regressions.

ABI metadata support alone does not make a type externally dispatchable. Keep
ABI JSON, selector spelling, calldata decoding, and result encoding aligned.

## Verification

After every std update, verify at least:

```sh
for file in ABIGeneric.sol Generic.sol StorageGeneric.sol dispatch.sol \
  eip712.sol eip7951.sol opcodes.sol std.sol; do
  cmp "std/$file" "crates/parser/tests/fixtures/corpus/ok/std/$file" || exit 1
done
cargo test -p solcore-parser -p solcore-hir-ty -p solcore-specialize --locked
E2E=1 E2E_REQUIRED=1 cargo test --profile e2e \
  -p solcore-yul --test e2e --locked -- \
  --nocapture --test-threads=1
```

Raw-vector cases retain their calldata, output, and EVM metadata; fixtures may
append regressions. Both Yul and Sonatina compile
and execute the complete set against Osaka. See
[`tests/e2e/README.md`](../tests/e2e/README.md) for the full validation
commands.
