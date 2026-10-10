# Haskell/Rust semantic compatibility and standard-library policy

This is the current compatibility decision record. Test verdict ledgers are
observations of particular compiler modes, not the language specification.

## Comparison target

- Haskell: [`argotorg/solcore@a7b7afb0`](https://github.com/argotorg/solcore/tree/a7b7afb0638e50895008d778a167afd0e50a2c86),
  the upstream HEAD checked on 2026-10-10.
- Starting Rust revision: `ee0bc4fb`.
- All eight `.sol` files in [`std/`](std/) and
  [`corpus/ok/std/`](crates/parser/tests/fixtures/corpus/ok/std/) are byte-for-byte
  copies of that upstream revision.
- Both compilers now use the canonical
  [source syntax](https://github.com/argotorg/solcore/blob/a7b7afb0638e50895008d778a167afd0e50a2c86/doc/src/sail/syntax.md).
  The older Haskell spellings are not a compatibility target.

The 499-source parser corpus and `reference-frontend.tsv` deliberately remain
historical `2f372bde` evidence. That ledger used the legacy resolver with
specialization and generated dispatch disabled (`-n -g`): 337 passes, 160
failures, and two timeouts. Its example sources have not been relabeled as a
fresh latest-upstream corpus. The shared std used by Rust regression runs is
now newer; see the [corpus README](crates/parser/tests/fixtures/corpus/README.md).

## Changes synchronized from the latest reference

| Area | Rust behavior and regression coverage |
| --- | --- |
| Standard library | Exact current std, including `msgSender`, `msgValue`, `std.opcodes.revert_`, typed bindings, and the rewritten `storeBytesFromMemory`. `msg-context` preserves upstream raw vectors; `std-storage-bytes` covers dirty input padding and shrinking across 31/32/33/63/64/65-byte boundaries. |
| Structs | Named struct fields lower through the existing product ADT/Generic machinery. Local receivers shadow same-named types/modules, including nested field reads. Direct/imported/generic/nested field reads and direct contract-field updates are covered by frontend fixtures, UI diagnostics, `struct-fields`, `struct-shadowed-namespace`, `struct-storage`, `struct-compound`, and the exact upstream `struct-storage-updates` execution fixture. |
| Tuple patterns | Source tuple arity preserves a nested final field rather than flattening it into extra constructor arguments. Contextual typing and exhaustiveness agree for `(x, rest)`, `((), (x, y))`, and enum/struct tuple fields. An unbound generic product tail can expand to the remaining source fields, and nested aliases resolve during tuple splitting. Dir/UI fixtures and `tuple-patterns` / `tuple-open-tail` cover these cases. |
| Derived variant ABI | Eligible ADTs with two or more constructors use one `keccak256("Constructor(field signatures)")` tag followed by that constructor's payload. Dynamic values retain one offset to the tagged body. Compiler-derived encoders/decoders and std helpers agree. Updated upstream raw vectors cover arrays, dynamic variants, storage, and direct encoding. |
| Array literals | Syntax loads the canonical std runtime without opening source names. Generated allocation/initialization/storage calls bind to canonical definitions despite aliases, hidden names, or same-named locals/functions. Specialization follows the selected superclass dictionary for `Typedef` methods. Dir tests, Hull UI fixtures, and `array-implicit` runtime memory checks cover these paths. |
| Compile-time memory | An unresolved memory read cannot establish comptime purity, including through a wrapper or closure. Residual assembly and unknown call effects invalidate earlier heap facts, including inside branches, loops, and compound arguments. Actual evaluation can still fold a known `mstore`/`mload` sequence. UI fixtures cover stale and unresolved reads; evaluator tests cover successful folding. |
| Compile-time numeric values | A known numeric comptime binding can be used as a runtime value without leaking its erased type marker. Constructor parameter types follow materialized known arguments while preserving checks on unknown values. The Hull UI fixture and `comptime-materialize` cover decimal, hexadecimal, and mixed runtime/comptime constructor fields. |
| Assembly substitution | Constants are substituted in source order with lexical shadowing. Loop initializers see outer bindings before introducing locals, repeated writes invalidate loop constants, and untouched source constants survive residual assembly. The latest upstream `asm_subst` vectors and the additional `asm-scope` / Hull UI fixtures cover the behavior. Yul function scope remains separate. |
| Contract naming and nested storage reads | Same-named methods in separate contracts remain distinct. The latest upstream duplicate-method fixture and field-access-in-tuple runtime case pin the behavior. |
| Canonical syntax fixes | Upstream now accepts `let` in a `for` post clause and separates source Yul from internal quasiquotes. These old `2f372bde` differences are resolved; Rust retains its existing parser behavior and diagnostics. |
| Type names and witnesses | `word` is the builtin spelling; uppercase `Word` needs an explicit alias. Duplicate ADT/contract names in the same type scope are rejected. `@T` selects an available `Proxy<T>` from its scope or expected type and diagnoses a missing definition instead of inventing an unconstrained type. A loaded canonical std supplies the witness even through a selective import that does not open `Proxy`; custom witnesses still require constructor visibility. UI and dir fixtures cover absent, local, qualified, renamed, and selective-import witnesses. |
| Trait derivation | Latest upstream unary derivation and qualified/aliased trait selection are covered by dir fixtures. Parameterized contract `main` examples are adapted to module-scope functions so the tests isolate derivation rather than the invalid runtime entry convention. |

Struct field access observes constructor visibility, including through aliases
and qualified imports. Compound storage updates reuse the saved receiver;
`struct-load-once` distinguishes their single custom load from explicitly
written extra reads. These checks preserve the reference's abstraction and
evaluation rules.

## Intentional corrections and retained differences

Upstream defects discovered or reconfirmed during this audit are also recorded
in the local, deliberately unstaged `issues.md`. They are not requirements to
reproduce incorrect behavior.

| Area | Latest reference behavior | Rust decision |
| --- | --- | --- |
| Unknown variant tag | `buildABIDecodeSum` checks all but the last constructor and otherwise decodes the last variant. | Check every tag and revert on an unknown value. `abi-sum-roundtrip` tests invalid tags for static and dynamic two-constructor types. |
| Static variant head size | `buildABIAttribs` delegates to nested structural sums even though the derived encoder now emits only one tag. For three or more constructors this can count extra tag words. | Use one tag plus the maximum constructor-product head. The static 0/1/3-field variant regression requires exactly 128 bytes. |
| Definition-side variant signature evidence | Generated tags call `SigString` from `std.dispatch`; importing only `std.ABIGeneric` does not provide it. | Require definition-side `std.dispatch` evidence and issue a structured diagnostic. Never silently fall back to old structural tags. |
| Non-callable values | Latest legacy full frontend still accepts `Uncurry` and `rec`, which call a `word` value. | Reject non-callable values. |
| Function tuple arity | Function lowering erases the distinction between one tuple argument and several arguments. Latest `ListModule`, `Pair`, and `Peano` accept calls inconsistent with their explicit function types. | Preserve source parameter arity. `function_tuple_parameter_arity` rejects `f(x, y)` when `f` requires one tuple, while accepting `f((x, y))`. |
| Explicit closure desugaring | `compose_desugared` is rejected because its generated-style `invoke` implementation is considered insufficiently polymorphic. | Accept the valid explicit closure representation; retain the specialization regression. |
| Renamed type witnesses | With `import {Proxy as P} from lib`, `@word` is still annotated as unqualified `Proxy<word>`, which fails to unify with the expected `P<word>`. | Resolve the expected alias to the actual ADT and construct its visible `Proxy` constructor. `proxy_local_and_qualified` includes a renamed import. |
| Assembly lexical scopes | Deeply nested let names can incorrectly conceal writes to outer variables, yielding stale constants. Loop initializer masking and later assembly blocks can also retain erased comptime identifiers. | Track writes in their actual scopes, substitute initializers before binding names, and preserve untouched outer constants. `asm-scope` executes both conditional-write paths. |
| Compile-time heap invalidation | A runtime memory write can leave an older compile-time heap value available to a later comptime load. | Invalidate heap facts after residual memory effects. `ct_memory_after_runtime_asm` rejects direct, wrapped, branch, loop, and compound-expression witnesses. |
| Effectful call arguments | Inlining an unused parameter can discard its runtime argument's side effect, including a write before a later memory read. | Keep calls whose arguments retain write effects. `ignored-call-effects` verifies that an ignored argument executes exactly once per call. |
| Instance signatures | Latest legacy full frontend accepts the narrowed `MemoryType.size` implementation in `ixa`. | Keep the instantiated class-signature check (`SC0221`). |
| Runtime `main` | Any contract-local `main` suppresses dispatch, including a function with parameters. | Keep suppression, but require zero parameters (`SC0236`). The runtime has no argument source for this entry. |
| ABI validation | Nullary names pass through metadata; unsupported shapes can reach a partial Haskell `error`. | Validate actual ABI evidence and use structured diagnostics. Reject same-named user-defined substitutes for canonical std wrappers and unsupported `memory<DynArray<address>>` outputs in `storage_array` and `ufcs_array`. |
| Parameterized ADT metadata | Runtime evidence can be derived but `abiTypeOf` still handles source ADTs only as `TyCon n []`. | Keep the deliberate concrete source spelling extension, e.g. `Point<uint256>`, and require all concrete/phantom arguments to satisfy ABI evidence. |
| Signature/selector collisions | No equivalent upstream preflight. | Reject duplicate signatures and distinct signatures sharing four selector bytes. |
| Tuple return boundaries | Latest metadata preserves a single tuple return item, but the erased result passed to `abi_encode` does not preserve the corresponding outer dynamic-tuple offset. | Retain flattened output metadata consistent with runtime bytes. Correct the source boundary, wrapper, metadata, and encoding together before changing this. |
| Nested tuple ABI boundaries | Right-nested products erase some source tuple boundaries. | Preserve the existing flattened ABI convention. This requires a coordinated language/ABI change, not a std-only patch. |

For tuple returns, a concrete upstream witness is
`returns ((uint256, memory<bytes>))`: metadata reports one dynamic tuple, while
`std.dispatch.runMethod` encodes the pair directly. ABI consumers expecting the
advertised outer tuple would interpret the first returned integer as an
offset. Rust keeps the same payload with two output entries. Static and nested
tuple execution remains covered by `composite-values`.

## ABI support and invariants

An externally supported type must have compatible metadata, selector spelling,
argument decoding, and result encoding. Merely printing a type in ABI JSON is
not support. Current runtime evidence is:

| Source type | Input | Output | Notes |
| --- | --- | --- | --- |
| `uint256`, `address`, `bytes4`, `bytes32`, `bool` | yes | yes | Boolean decoding is strict. |
| `memory<string>`, `memory<bytes>`, unit, supported tuples | yes | yes | Tuple boundaries follow the convention above. |
| Primitive `word` | no | no | std still lacks `SigString`, `ABIEncode`, and `ABIDecode` instances even though metadata spells `uint256`. Missing evidence terminates with a bounded diagnostic. |
| Canonical `calldata<array<T>>` | yes | no | Lazy calldata handle; requires element input evidence. This restriction applies inside nested ADT results too. |
| Finite compiler-derived ADT | yes | yes | Requires the defining module's derivation markers and complete leaf/type-argument evidence. Multi-constructor ADTs use variant tags. |
| Recursive, excluded, or manually represented ADT | no | no | No compiler-owned finite decode/metadata plan. |

Selectors continue to use structural Generic `SigString` spellings, such as
`sum(uint256,bytes)`, while ADT ABI JSON retains source names. **Selector
spelling and constructor wire tags are different**: the new tag is the full
32-byte keccak hash of the constructor name and its field signatures. This
does not claim general Solidity-tool support for arbitrary source ADT names.

Explicit structural `sum` values and `std.ABIGeneric.encode`/`decode` still use
the original Generic 0/1 representation. Only the compiler-derived ADT
`ABIEncode`/`ABIDecode` path uses constructor tags. Product ADTs keep their
structural product layout.

Manual `SigString`, `ABIAttribs`, `ABIEncode`, and `ABIDecode` instances remain
usable in ordinary code and hand-written fallback dispatch. Rust rejects them
when selected by generated external ABI evidence. The check follows evidence
chosen in both consuming and defining modules, including selector and variant-tag
signatures and the actual constructor-product dependencies of derived encoders
and decoders. An output-only ADT's own unused `SigString` instance and unrelated
instances do not cause rejection.

Top-level `abi_encode` returns length-prefixed `memory<bytes>`, and generated
dispatch returns only its payload. These two std behaviors remain paired.
Derived storage remains definition-scoped and supports finite ADTs, mapping
values, and dynamic bytes/string leaves. Whole mapping-valued leaves and
recursive derived storage remain rejected.

## Resolver modes and validation scope

Haskell defaults to legacy resolution. Rust uses tabled resolution; compare
like modes before calling a discrepancy a language difference. The old
`super-class-recursive-arg`, `tabled-answer-reuse`, and `tabled-mutual-chain`
cases succeed with Haskell tabled resolution and Rust.

Conversely, latest Haskell tabled resolution rejects `msg_context`,
`field-access-in-tuple`, `arrays/shadowed`, and `new-syntax/struct-storage` with
unresolved `CanStore` obligations. Latest legacy full frontend accepts all four.
Rust accepts them. These are recorded solver defects rather than reasons to
remove working Rust behavior.

`ct_param_poly_runtime` is a phase difference: the Rust frontend preserves a
latent comptime obligation, and specialization rejects its runtime argument.
The Haskell legacy frontend first reports ambiguity; tabled full compilation
also rejects the runtime argument. A frontend-only pass does not mean runtime
values are allowed as comptime arguments.

The current audit compared all 518 `test/examples` sources and all 37
`test/new-syntax` sources with specialization disabled and dispatch enabled.
The Haskell reference was the existing local build from the same clean tracked
revision. Parser-only reference examples are not necessarily valid complete
programs; missing imports, intentionally negative declarations, or parameterized
runtime mains must not be reclassified as expected full-pipeline successes.
Focused dir/UI and execution regressions cover implemented changes. The
historical ledger's allowances remain separately labeled by solver and phase;
old `-g` successes do not establish generated-dispatch or backend compatibility.

The final frontend audit used Haskell's legacy resolver and a 15-second
per-source Haskell timeout. Its results on the 555 current sources were:

| Haskell | Rust | Sources |
| --- | --- | ---: |
| pass | pass | 353 |
| fail | fail | 182 |
| pass | fail | 13 |
| fail | pass | 5 |
| timeout | fail | 2 |

The 13 Rust rejections are the documented function-arity, non-callable-value,
instance-signature, runtime-main, and unsupported-ABI cases. The five Rust
acceptances are explicit closure desugaring, three resolver-mode differences,
and the deferred comptime obligation. Haskell timed out on `tabled-cycle-fail`
and `tabled-left-recursive-fail`; Rust returned bounded diagnostics. These
frontend counts do not establish runtime correctness by themselves.

The follow-up review reran the Rust frontend on all 555 sources after the
scope, tuple, witness, and evaluator fixes above; these verdict counts were
unchanged. Additional probes checked generic/nested variant ABI layouts and
definition-side signature evidence, and 15 ABI round trips executed
successfully. The newly found evaluator and name-resolution cases are pinned
by the focused UI/dir and shared backend E2E fixtures described above.

Use the existing workflows for verification:

```sh
for file in ABIGeneric.sol Generic.sol StorageGeneric.sol dispatch.sol \
  eip712.sol eip7951.sol opcodes.sol std.sol; do
  cmp "std/$file" "crates/parser/tests/fixtures/corpus/ok/std/$file" || exit 1
done
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
E2E=1 E2E_REQUIRED=1 cargo test --profile e2e \
  -p solcore-yul -p solcore-sonatina --test e2e --locked -- \
  --nocapture --test-threads=1
```

## Standard-library update policy

Prefer byte-exact upstream library sources and record their full revision.
Copy updates into both std locations together. When upstream has a confirmed
bug or specification defect, document the witness and the chosen correction;
there is no requirement to reproduce that defect or obtain an upstream fix
before preserving correct Rust behavior. Compiler-side corrections should be
kept explicit. If a library deviation becomes necessary, document it per file
instead of continuing to claim byte-exact synchronization.
