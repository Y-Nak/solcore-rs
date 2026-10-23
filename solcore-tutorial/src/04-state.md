# Contracts and persistent state

**Goal:** distinguish a local binding from contract storage, and a test assertion from a committed state change.

<div class="tutorial-exercise" data-title="Persistent counter">
<a class="tutorial-example" href="../projects/tutorial-counter.json?tab=run&amp;view=tests">Open Persistent counter in a full tab</a>
</div>

```solidity
{{#include ../examples/tutorial-counter/Counter.sol}}
```

`count` is a contract field. Its value survives calls to the same deployed instance. The constructor runs once per deployment and initializes it to `0`. `add` updates that field; `read` returns it. A new instance starts at zero again.

## Explore one instance

Open **Try calls** and use **Reset contract** if a session already exists. Select each function, set its arguments, and click **Call function** in this order:

| Step | Function | Arguments | What to observe |
| --- | --- | --- | --- |
| 1 | `read()` | `[]` | `0`; the first call also deploys the contract |
| 2 | `add(uint256)` | `[2]` | Successful call; this function has no return value |
| 3 | `add(uint256)` | `[3]` | Another successful state change |
| 4 | `read()` | `[]` | `5` |

Here `[]` means no arguments. The playground hides the Arguments field for such functions; simply click **Call function**.

Now use **Reset contract** and call `read()` again: it returns **0**. Editing source or switching examples also resets the manual instance. Manual state is not a permanent blockchain deployment.

## Test a sequence

Click **Run all tests from start**. The two `// #[send(...)]` directives commit changes to the separate test instance, in source order. The final `// #[() -> 5]` checks the resulting state. All **three** steps should pass.

A normal expectation runs a call but discards its state changes afterward. A `send` persists changes for later test steps; it does not assert a returned value. That is why this example sends twice and then reads. Running the final assertion by itself replays the preceding sends, so it still expects `5`.

The tests do not change the manual instance from **Try calls**. If your manual counter was reset to zero, it remains zero after the tests.

## Change it

Change the first send from `2` to `4`. Predict the final value, update the read expectation, and rerun all tests. Then run only the final test.

<details>
<summary>Hint and answer</summary>

Use `// #[send(4)]` followed by the existing send of `3`, and change the expectation to `// #[() -> 7]`. All tests pass; the final individual test also returns `7` because its setup sends are replayed.

</details>

**Check your understanding:** why would replacing a `send` with a normal assertion fail to prepare the next test's storage?
