# Solcore standard library

The ten `.sol` files in this directory are the standard library for the Rust
compiler. Format them with `solcore-fmt`. Parser, frontend, specialization, and
backend tests read these same sources directly.

## Maintenance policy

Maintain the library alongside the Rust compiler. Update and format each
changed source. Cover language and ABI changes with focused frontend,
specialization, and backend regressions.

ABI metadata support alone does not make a type externally dispatchable. Keep
ABI JSON, selector spelling, calldata decoding, and result encoding aligned.

## Integer types and checked arithmetic

`signed.sol` provides `int8` .. `int256` and `unsigned.sol` provides `uint8` ..
`uint128` (`uint256` is in `std.sol`). They are accepted as external ABI types
(`intN`/`uintN`). Arithmetic on them and on `uint256` is checked as in
Solidity 0.8: overflow, underflow and `MIN / -1` revert with `Panic(0x11)`,
division or modulo by zero with `Panic(0x12)`. The per-type instances delegate
to the `uAddChecked` .. `sModChecked` helpers in `std.sol`, whose conditions are
bound to `let` locals rather than nested calls to keep call nesting within the
default parser stack. Regressions: `tests/e2e/int-ops` (ABI decode/encode and
storage), `tests/e2e/checked-arith`, and
the shared matrix in `crates/test-utils/src/e2e/integers.rs` (433 arithmetic,
bitwise, comparison and bounds cases across all 11 added integer types).
Both backends generate its source fixtures at test time and execute them
against independently calculated results.

Integer defaulting selects the canonical `uint256` type. Arithmetic and compound
assignments therefore remain checked without a type annotation. Programs using
this default must import the standard library; an unavailable default type is
an error. Explicit `word` arithmetic retains the low-level EVM wrapping behavior.

Unary `-x` uses checked subtraction from zero. Negating a signed type's minimum
value panics with code `0x11`; negating a nonzero unsigned value also overflows.
Negative literals, including signed minima, use the type's `Int.fromInteger`
conversion. In a required `comptime` evaluation, a failing arithmetic check is
a compile error.

Literals and `Int.fromInteger` conversions for all these integer types are checked
against the type's bounds at compile time, before conversion to a 256-bit
word. `Typedef.abs` remains a raw representation conversion. Explicit `word`
literal conversions retain their modulo-2^256 behavior.

ABI decoding rejects noncanonical upper bits with `DirtyHigherBitsForInteger()`;
encoding masks unsigned values and sign-extends signed values to their declared
width, including raw values constructed through `Typedef.abs`. The shared ABI
matrix in `crates/test-utils/src/e2e/integers/abi.rs` covers all 11 types through
calldata and memory, including tuples, arrays, and malformed input.

## Verification

After every std update, verify at least:

```sh
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
