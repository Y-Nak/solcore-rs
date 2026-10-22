# Build a bounded counter

**Goal:** combine persistent state, a condition, rejection, and tests in a small feature you can explain.

Start from [the Counter example](https://playground.solcore.soliditylang.org/#/examples/tutorial-counter?tab=run&view=tests). Keep its constructor, `add`, and `read`. Your counter should accept a total of at most **10** units. Adding more than the remaining capacity must revert, without changing the count. Add a public `remaining()` function returning `uint256`.

This is a numeric exercise, not a payment or token contract. Any caller may add; access control is outside its scope.

## Write the behavior first

Use these checks as your target, replacing the starter's test comments:

| Action | Expected behavior |
| --- | --- |
| Send `add(4)` | Accepted |
| Send `add(6)` | Accepted; the count reaches the bound |
| Assert `add(1)` reverts | Rejected |
| Assert `read()` returns `10` | Rejection preserved the existing value |
| Assert `remaining()` returns `0` | No space remains |

Keep the sends and rejected-call expectation above `add`, and the read expectations above their respective functions. Run all tests from the start after each change.

## Think about arithmetic before coding

Current `uint256` addition wraps. Testing only whether `count + amount > 10` could allow an overflowing addition to escape your check. Instead, maintain the invariant **`count <= 10`** and compare `amount` with `10 - count` before adding.

Explain why the subtraction is valid: the constructor starts at zero, and every accepted update stays within the bound. No other function should write an arbitrary value into `count`.

<details>
<summary>First hint</summary>

Use an `if` at the beginning of `add`. If `amount > uint256(10) - count`, call `revertWithError("capacity exceeded")`. Otherwise keep the original assignment. `remaining()` returns `uint256(10) - count`.

</details>

<details>
<summary>Complete, tested solution</summary>

Save any experiment you want to keep before switching examples.

[Open in playground: Bounded counter solution](https://playground.solcore.soliditylang.org/#/examples/tutorial-bounded-counter?tab=run&view=tests)

```solidity
{{#include ../../playground/src/examples/tutorial-bounded-counter/BoundedCounter.sol}}
```

All **five** test steps should pass. The expected revert is itself a passing test when rejection occurs. The subsequent read verifies that the count is still `10`.

</details>

## Try your own cases

Use **Reset contract** if shown, then call `remaining()` and expect `10`. No Arguments field is shown for this no-argument function. Call `add` with `[11]` and expect a revert, then call `read()` and expect `0`. Call `add` with `[0]` and verify that it succeeds without changing the count. These cases explore the empty, oversized, and zero-input boundaries independently of the test sequence.

**Check your understanding:** why does this guard remain correct for very large `uint256` inputs? What extra requirement would you need if only a particular account should be allowed to increment the counter?
