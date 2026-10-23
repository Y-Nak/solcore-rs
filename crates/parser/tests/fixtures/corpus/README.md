# Solcore frontend regression corpus

This corpus contains 499 historical examples under `test/examples/`, preserving
their module layout while using the canonical `.sol` syntax. Parseable sources
are formatted with `solcore-fmt`; intentional syntax-error fixtures remain
unchanged. The `ok/test/examples/` and `fail/test/examples/` directories retain
their historical classifications rather than defining current language behavior.

The standard-library sources in `ok/std/` are byte-identical to
[`std/`](../../../../../std/); see
[`std/README.md`](../../../../../std/README.md) for the synchronization
policy. The `test/imports/` and `known-diagnostic-gaps/` trees contain additional
Rust compiler regressions outside the historical example set.

The Rust compiler and its tests define current language behavior. The historical
ledger is a fixed regression baseline and is not a current compiler verdict.

## Historical baseline

[`reference-frontend.tsv`](reference-frontend.tsv) records 337 passes, 160
failures, and two timeouts from the historical baseline. Ledger paths map to
the `.sol` files by module stem. `code` is the
first structured `SCnnnn` diagnostic emitted for a failure; `-` means that no
structured code applies. The two timeout rows are
`cases/tabled-cycle-fail.sol` and `cases/tabled-left-recursive-fail.sol`.
The two allowance manifests record how the Rust frontend gates treat that
baseline:

- [`rust-accepted-reference-failures.tsv`](rust-accepted-reference-failures.tsv)
  lists historical failures accepted by the corresponding Rust frontend gate.
- [`rust-rejected-reference-passes.tsv`](rust-rejected-reference-passes.tsv)
  lists diagnostics from Rust's full-frontend gate for historical passes. It
  includes deliberate negative import fixtures and phase differences
  caused by enabling generated dispatch.

Add new regressions to dedicated test/fixture trees rather than the historical
example trees. Parser snapshots are retained only for historically failing
sources that also exercise a Rust parser diagnostic.
