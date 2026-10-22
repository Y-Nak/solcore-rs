# Representing state with enums and match

**Goal:** In this chapter, group the data needed by each state into an `enum`, then use
`match` to inspect it. Building on the previous chapter's storage, model a
transition from awaiting payment to funded to released.
This `Escrow` records numbers for learning. It does not receive or transfer
Ether, or restrict who can call its methods.

[Open Escrow in the Playground](https://playground.solcore.soliditylang.org/#/examples/pattern-matching?tab=run&view=tests)

```solidity
{{#include ../../playground/src/examples/pattern-matching/Escrow.sol}}
```

`AwaitingPayment` carries no data. `Funded(uint256)` and `Released(uint256)`
each carry a recorded amount. The declaration shows which data belongs to
which state, instead of keeping a numeric state flag and amount separately.
`case Phase.Funded(amount)` checks the variant and binds its payload to
`amount`. `_` ignores a payload; `default` handles cases not matched earlier.

`phase` lives in contract storage. The `std.Generic` and
`std.StorageGeneric` imports provide the derivation machinery used to store
this user-defined type. An enum declaration does not mean every possible
user-defined type can be stored without restrictions.

## Run and observe

Choose **Run → Run tests → Run all tests from start**. All 6 tests pass.
After the `deposit(42)` send, a second deposit reverts. The release check
returns `42`, and the final status is `2`. An ordinary expected-value test
does not commit state changes: that is why release has a `#[send()]` after
its `#[() -> 42]` check.

To inspect the transitions yourself, open **Try calls** and use **Reset contract** if it is shown,
then select each function and press **Call function** in this order:

| Function | Arguments | Result |
| --- | --- | --- |
| `status()` | `[]` | `0` |
| `deposit(uint256)` | `[42]` | Success |
| `status()` | `[]` | `1` |
| `release()` | `[]` | `42` |
| `status()` | `[]` | `2` |
| `release()` | `[]` | Revert |

For a no-argument function, `[]` in the table means there is no Arguments field to fill in.

The last revert is intentional. The contract calls `revertWithError` when
the requested transition is not allowed.

## Exercise

Add `releasedAmount()`: return the recorded amount in the `Released` state,
and `0` in every other state. Place its test after the existing `status`
method, so the test runs after the release send.

<details>
<summary>Hint and solution</summary>

Use `status()` as a guide. Inside the same contract, add a public function
named `releasedAmount`, with no parameters and a `uint256` return type.
Its body matches on `phase`: in `case Phase.Released(amount)`, use
`return amount;`; in `default`, use `return 0;`.
Place `// #[() -> 42]` immediately above the new function, and keep that
function after `status` so its check sees the preceding setup sends.

**Run all tests from start** now passes 7 tests. Editing the source resets
the manual contract session, so repeat deposit and release if you want to
check this function through Try calls.

</details>

**Check your understanding:** why does the release expectation need a separate send before the final status check?
