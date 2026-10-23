# Your first Solcore program

**Goal:** run a contract and identify its imports, entry point, and return value.

<div class="tutorial-exercise" data-title="Hello contract">
<a class="tutorial-example" href="../projects/contract-output.json?tab=run">Open Hello contract in a full tab</a>
</div>

```solidity
{{#include ../examples/contract-output/Hello.sol}}
```

## Run it

1. Click **Try here** above to open the editor within this page. It uses the **solc** backend by default.
2. In the editor, open the **Run** tab, then click **Run** for `main`.
3. Find the returned word: **42**. A successful compilation alone is not an execution result.

The first run can take longer while the browser loads the compiler. This program returns a value rather than printing text to a terminal. Gas used describes execution work; its exact value can change with the compiler and backend, so it is not the goal of this exercise.

## Read it

`import * from std;` makes the standard library's exported names available, including `uint256`. `std.dispatch` provides the machinery used for external contract interfaces. Keep both imports in the contract examples that follow.

`contract Hello` groups contract code. `function main()` has no arguments. `public` makes this a public runtime entry, and `returns (uint256)` states its result type. `return uint256(42);` constructs and returns the value. Statements end with semicolons.

`main` is special: this example uses an explicit runtime entry. Later examples omit it so the compiler can generate a dispatcher that selects public functions by their ABI signatures. Do not add a `main` to those examples merely because Hello has one.

## Change it

Make this program return **7**, then run it again. After that, deliberately remove the semicolon after the return statement and inspect the **Problems** tab. Restore the semicolon before continuing.

<details>
<summary>Hint and answer</summary>

Replace `uint256(42)` with `uint256(7)`. The returned word should be `7`. Removing the semicolon produces a syntax diagnostic; the exact wording may evolve, but the source location should help you find the mistake. A failed compilation produces no new successful execution.

</details>

**Check your understanding:** what is the difference between clicking **Compile** and **Run**? Compile produces artifacts and diagnostics; Run also executes the entry point.
