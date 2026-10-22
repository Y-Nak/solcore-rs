# Teaching design and references

This tutorial uses the current repository's language implementation and
Playground examples as its baseline. Other language tutorials inform the
sequence of explanation, execution, and practice. Their syntax and language
semantics are not specifications for Solcore.

## Reference tutorials

| Reference | Useful pattern | Application here |
| --- | --- | --- |
| [The Rust Programming Language](https://doc.rust-lang.org/book/) | A book progressing from basics toward types and abstraction | Chapter sequence, prerequisites, and learning objectives |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) and [Hello World](https://doc.rust-lang.org/rust-by-example/hello.html) | Modifiable executable examples followed by activities | Existing `.sol` examples, small changes, and expected results |
| [A Tour of Go](https://go.dev/tour/welcome/1) and its [official introduction](https://go.dev/blog/tour) | Playground-backed editing, compilation, execution, and exercises | A browser-first path before installing a local toolchain |
| [Svelte Tutorial](https://svelte.dev/tutorial/svelte/welcome-to-svelte) | Explicit prerequisites and sequence, exercises, and solutions | Concrete steps and collapsible hints and solutions |

References consulted on October 22, 2026. The prose and exercises were
written for the current Solcore examples.

## Why use the existing Playground?

The Playground already supports compilation, browser-side EVM execution,
expected-value tests, and manual calls that retain state. Readers can also
inspect multiple files, diagnostics, and ABI output. It lets them check a
claim immediately using the same compiler and examples as the development
Playground.

| Integration | Benefit | Decision |
| --- | --- | --- |
| Direct links to an example, file, and view | Works in Markdown readers and gives the IDE a full window | Primary learning path |
| Always-visible iframe | Explanation and editor appear together | Not used initially: the full IDE needs space and adds initial loading work |
| Iframe loaded on request | Defers loading while keeping practice on the page | A future option with separate persistence and a compact layout |
| Dedicated tutorial editor | Can manage exercise starters, solutions, and progress | Deferred to avoid duplicating existing compiler and editor integration |

This is a judgment based on the current implementation. An iframe sharing
the normal Playground's origin also shares its localStorage, so opening a
lesson can affect saved code. A future embedded version should be opened
explicitly, use a separate workspace storage key, and retain an option to
open a full tab.

## Reproducible practice

Links identify bundled examples. They do not share edited source, manual
arguments, or contract state. Opening the same example again preserves
saved edits; opening a different example replaces the current workspace.
Save code you want to retain before moving to another example.
**Reset workspace** restores the initial Hello example. To restart a
lesson from its original code, reset and then reopen that lesson's link.

Tests and manual calls are explained separately. Tests start from a new
deployment for each contract and replay preceding sends. Manual calls
retain their session state. Understanding that these states are independent
helps explain why a manual call may differ from a test result.

## Keeping the tutorial aligned

Complete listings use mdBook includes to read the existing `.sol` files,
rather than maintaining a second copy. When an example changes, its
expected results and instructions must change too. Existing compiler tests
check the Playground sources; WASM tests exercise the principal examples
and their expected values. Tutorial links and includes are also checked
when generating the tutorial.

The public Playground is deployed separately, so it is not guaranteed to
match a local checkout. If its syntax or features differ from this text,
build the Playground from this repository. The tutorial README describes
generation and local use.
