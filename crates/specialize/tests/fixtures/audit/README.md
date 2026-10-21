# Specialization regression fixtures

`tests/support/audit_fixtures.rs` discovers every `*.case.json` under
`tests/fixtures` with `dir-test`. Each JSON file selects a source relative to its
own directory and declares semantic expectations. Multi-module cases keep their
existing sources and attach one expectation file to each entry module. Shared
E2E sources are referenced directly rather than copied.

The loader is `bare` for isolated source, `std` for the reference standard
library, `std_current` for the workspace standard library, or `fixture` for an
existing file and its sibling modules. `max_depth` preserves explicit recursion
limits. `frontend_clean`, `diagnostics`, and `forbidden_diagnostics` independently
describe frontend and specialization verdicts. An empty diagnostic list requires
successful specialization; a nonempty list requires each named diagnostic,
optionally matching its context.

Checks inspect Mono IR directly: constant and variable returns, call signature
materialization, residual matches and declarations, closure erasure, function
counts, selected dictionary results, and function parameter/return identities.
Function selectors describe names or method origins; no fixture name selects a
bespoke assertion routine. Unknown JSON properties are rejected.

Numbered source cases retain the individual variants previously combined in a
single Rust test. Each variant is now separately discovered and reported.
Ordinary `.sol` files use `solcore-fmt` formatting. The `.invalid.sol.txt` input
deliberately retains an invalid parameter annotation and is loaded verbatim;
formatting it would reject the input before its diagnostic checks could run.

Direct evaluator tests in `src/evaluate/pattern_tests.rs` remain unit tests:
they construct intermediate expression matches and partially known values to
exercise evaluator invariants independently of source lowering. The type
substitution unit test likewise checks internal qualified type identity and
substitution state directly. Source fixture tests complement these invariants.
