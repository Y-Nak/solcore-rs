# The Solcore Tutorial

Learn to write a small Solcore contract, run it in your browser, and explain why it behaves as it does. You will start with a function returning `42`, then work through values, tests, persistent state, enums, modules, traits, and compile-time computation. The final exercise adds a bound to a counter and checks both successful and rejected calls.

## Who this is for

You should already be comfortable with variables, functions, and reading short programs in some language. You do not need Rust, Solidity, a wallet, or a local blockchain. We introduce the contract concepts as they become necessary. Allow about two to three hours for a first pass, including the exercises; each chapter can also be studied separately.

Solcore is an experimental language targeting the Ethereum Virtual Machine (EVM). This tutorial follows the **current `solcore-rs` implementation and its checked-in standard library**. The `.sol` extension does not mean that these programs are Solidity. Familiar Rust ideas such as enums and traits help with comparisons, but Rust's ownership, borrowing, lifetimes, and `mut` rules are not the rules taught here.

> Solcore and these examples are works in progress, not production-ready contracts. The playground executes against an in-memory EVM; the exercises do not send real transactions or require real funds.

## Read → predict → run → change

Every chapter gives you a goal, an example, an expected result, and a small change to try. Predict the result before running it. The expandable hints let you get unstuck without reading the solution first. At the end of a chapter, explain the result in your own words before continuing.

Use the **Open in playground** links to load the complete example, including its helper files. They open a separate tab in the built book, so the explanation stays available. Select the **solc** backend for your first pass. The exercises are also tested with Sonatina.

The book bundled at the playground's `/tutorial/` path links to that same build. If you are reading the Markdown source or a standalone book build, links instead point to the public playground. That deployment may lag behind this checkout; use the [local tools](appendix-tools.md) when a new example is missing or behaves differently.

## Three ways to execute

| Action | Use it for | State behavior |
| --- | --- | --- |
| **Run** for `main` | The first Hello example | A fresh deployment on every run |
| **Run all tests from start** | Comments containing inputs and expected results | A separate fresh test deployment; `send` steps persist within it |
| **Call function** under **Try calls** | Exploring public functions manually | Changes persist between calls until reset or source changes |

Opening a link selects code and a view; it does **not** run the program. Tests and manual calls use separate contract instances. A passed test does not prepare the state of your manual calls.

## Keep your experiments

The playground saves one workspace in this browser. Opening a different example replaces that workspace; opening the same example may preserve your edits. Copy code you want to keep to a local file **before** moving to another example. A shared example URL does not contain your edits, arguments, or contract state.

To restore an example, choose **Reset workspace** (which returns to Hello), then reopen that chapter's example link. To keep your source but clear manual state, use **Reset contract** instead. Keep one active playground tab to avoid competing saved workspaces.

[Start with your first program →](01-first-program.md)
