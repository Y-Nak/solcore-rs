# Specialization regression fixtures

`tests/support/audit_fixtures.rs` discovers the audit cases with `dir-test`, using
one `main.sol` entry per case directory (`main.invalid.sol.txt` for deliberately
malformed source). Multi-module families select their
entry modules separately from the supporting imports. Shared E2E `main.sol`
sources are discovered directly rather than copied.

Typed Rust expectations live in `tests/support/audit_cases.rs`; source fixtures
have no companion metadata files. Expectations select isolated source, the
parser-corpus standard library, the workspace standard library, or an existing
file and its sibling modules. Explicit recursion limits are preserved where
needed. Frontend and specialization verdicts are checked independently. An
empty diagnostic list requires successful specialization; a nonempty list
requires each named diagnostic, optionally matching its context. Cases can also
forbid specific diagnostics.

Checks inspect Mono IR directly: constant and variable returns, call signature
materialization, residual matches and declarations, closure erasure, function
counts, selected dictionary results, and function parameter/return identities.
Function selectors describe names or method origins. Expectations and assertions
are checked by the Rust compiler alongside the fixture runner.

Numbered source cases retain the individual variants previously combined in a
single Rust test. Each variant is separately discovered and reported.
Ordinary `.sol` files use `solcore-fmt` formatting. A deliberately invalid
parameter annotation is loaded verbatim; formatting that input would reject it
before its diagnostic checks could run.

Direct evaluator tests in `src/evaluate/pattern_tests.rs` remain unit tests:
they construct intermediate expression matches and partially known values to
exercise evaluator invariants independently of source lowering. The type
substitution unit test likewise checks internal qualified type identity and
substitution state directly. Source fixture tests complement these invariants.
