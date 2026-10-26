# Specialization regression fixtures

`tests/support/audit_fixtures.rs` discovers audit cases with `dir-test`. Each
audit case has a `main.sol` entry (`main.invalid.sol.txt` for deliberately
malformed source). Entry modules declare their specialization expectations in
comments; supporting imports do not. Separate Rust tests scan the local fixture
and shared E2E directories for annotated entries, so adding an entry requires no
Rust registry. Unannotated files do not produce specialization test cases.

For example:

```solidity
//~ specialize: loader fixture
//~ specialize: frontend clean
//~ specialize: return 7
//~ specialize: no-closure-dispatch
```

The following expectations are supported:

- `loader bare`: specialize isolated source without a standard library.
- `loader std`: use the workspace standard library (`std-current` is an alias).
- `loader fixture`: load the source file, sibling modules, and workspace standard
  library. Every annotated entry must specify a loader.
- `frontend clean` or `frontend errors`: check the frontend independently of
  specialization. Omitting this leaves frontend diagnostics unchecked.
- `max-depth 16`: override the specialization recursion limit for a source loader.
- `diagnostic missing-evidence`: require a specialization diagnostic. Diagnostic
  names also include `free-type-variable`, `comptime-evaluation-failed`,
  `integer-erasure`, and `instantiation-depth-exceeded`. Append `= exact context`
  or `contains context fragment` to constrain its context. Multiple diagnostic
  comments require each diagnostic; no diagnostic comments require successful
  specialization.
- `forbid instantiation-depth-exceeded`: reject a particular diagnostic.
- `return 7` or `return none`: check the result of the main-return evaluator.
- `return-variable seed`: require the evaluated main return to reference a name.
- `entry-return 7` or `entry-return-variable seed`: check the runtime entry's last
  return directly.
- `entry`: require an entry point.
- `no-closure-dispatch`: require closure dispatch to have been eliminated.

Unknown expectations and duplicate loader, frontend, or depth settings fail the
test. Unannotated sources outside this audit directory are supporting modules or
belong to other suites. Every source in this audit directory requires annotations.

Tests in `tests/support/audit_ir.rs` inspect more complex Mono IR relationships
with ordinary Rust assertions: materialized call signatures, residual matches
and declarations, closure capture parameters, function counts, dictionary
results, and parameter/return type identities. These tests load the same source
fixtures and check their comment expectations before inspecting the IR.

Numbered audit source cases retain the individual variants previously combined
in a single Rust test. Each audit variant is separately discovered and reported.
Ordinary `.sol` files use `solcore-fmt` formatting. A deliberately invalid
parameter annotation is loaded verbatim; formatting that input would reject it
before its diagnostic checks could run.

Direct evaluator tests in `src/evaluate/pattern_tests.rs` remain unit tests:
they construct intermediate expression matches and partially known values to
exercise evaluator invariants independently of source lowering. The type
substitution unit test likewise checks internal qualified type identity and
substitution state directly. Source fixture tests complement these invariants.
