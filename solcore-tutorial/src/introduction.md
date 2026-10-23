# The Solcore Tutorial

Learn to write a small Solcore contract, run it in your browser, and explain why it behaves as it does. You will start with a function returning `42`, then work through values, tests, persistent state, enums, modules, traits, and compile-time computation. The final exercise adds a bound to a counter and checks both successful and rejected calls.

## Who this is for

You should already be comfortable with variables, functions, and reading short programs in some language. You do not need Rust, Solidity, a wallet, or a local blockchain. We introduce the contract concepts as they become necessary. Allow about two to three hours for a first pass, including the exercises; each chapter can also be studied separately.

Solcore is an experimental language targeting the Ethereum Virtual Machine (EVM). This tutorial follows the **current `solcore-rs` implementation and its checked-in standard library**. The `.sol` extension does not mean that these programs are Solidity. Familiar Rust ideas such as enums and traits help with comparisons, but Rust's ownership, borrowing, lifetimes, and `mut` rules are not the rules taught here.

> Solcore and these examples are works in progress, not production-ready contracts. The playground executes against an in-memory EVM; the exercises do not send real transactions or require real funds.

## Read → predict → run → change

Every chapter gives you a goal, an example, an expected result, and a small change to try. Predict the result before running it. The expandable hints let you get unstuck without reading the solution first. At the end of a chapter, explain the result in your own words before continuing.

Choose **Try here** in a chapter to open an editable example inside the page, including its helper files. The editor loads only when requested; **Close editor** releases it while keeping your saved code. Only one inline editor is open on a page at a time. Use **Open in full tab** whenever you want more space or the full Playground controls.

The distributed tutorial includes its own Playground, compiler, and example sources. Running a lesson does not contact the public Playground or send your code to a compilation service. Serve the complete tutorial folder over HTTP; after its files load, execution happens in your browser. See [local tools](appendix-tools.md) for build and serving commands. The tutorial owns the projects used by its lessons; they do not depend on the ordinary Playground example menu. Plain mdBook previews include the source listings but disable editing links until you build the complete tutorial.

The inline editor uses the **solc** backend by default. The exercises are also tested with Sonatina, which you can select in the full-tab view.

## Three ways to execute

| Action | Use it for | State behavior |
| --- | --- | --- |
| **Run** for `main` | The first Hello example | A fresh deployment on every run |
| **Run all tests from start** | Comments containing inputs and expected results | A separate fresh test deployment; `send` steps persist within it |
| **Call function** under **Try calls** | Exploring public functions manually | Changes persist between calls until reset or source changes |

Opening an editor selects code and a view; it does **not** run the program. Tests and manual calls use separate contract instances. A passed test does not prepare the state of your manual calls.

## Keep your experiments

Each tutorial project saves its source files in a separate workspace in this browser. Returning to that project restores your edits, and opening another lesson does not replace them. Chapters using the same project share its workspace. Project workspaces are separate from the ordinary Playground workspace. The inline editor and its **Open in full tab** view use the same saved source; keep only one editor open for a given project.

Choose **Reset project** to restore that project's original source. To keep your source but clear manual contract state, use **Reset contract** instead. Closing an editor or reloading the page also starts a new execution session; saved source is not saved contract state.

Browser storage is a convenience, not an export. Copy code you want to keep to a local file, especially before clearing site data or using a private window. A shared project URL identifies the bundled project; it does not contain your edits, arguments, or contract state.

[Start with your first program →](01-first-program.md)
