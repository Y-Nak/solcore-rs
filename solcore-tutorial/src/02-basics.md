# Values, functions, and control flow

**Goal:** follow a local variable through a helper call, an `if`, and a loop.

[Open in playground: Tutorial — Basics](https://playground.solcore.soliditylang.org/#/examples/tutorial-basics?tab=run&view=tests)

```solidity
{{#include ../../playground/src/examples/tutorial-basics/Basics.sol}}
```

## Predict and run

In the **Run** tab, select **Run tests** and click **Run all tests from start**. All five tests should pass:

| Function | Arguments | Result |
| --- | --- | --- |
| `adjust` | `21, true` | `42` |
| `adjust` | `21, false` | `21` |
| `adjust` | `0, true` | `0` |
| `sumBelow` | `0` | `0` |
| `sumBelow` | `5` | `10` |

The `// #[...]` lines describe these checks. You will learn how to write them in the [next chapter](03-tests.md).

## Values and local bindings

`uint256` represents an unsigned 256-bit value, and `bool` has the values `true` and `false`. `let result: uint256 = value;` introduces a local binding with an explicit type. In this program it exists only during the call. Solcore lets you reassign it without a `mut` keyword.

`double` is a top-level helper. Its parameter and return types are part of its declaration, and the contract calls it by name. It is not a public contract method. In `adjust`, only a true `bonus` enters the `if` body; both branches then reach the final return.

The standard library defines `uint256` around the low-level `word` type and supplies its operators through traits. Use `uint256` in these public interfaces: primitive `word` is not a supported public ABI type. Current `uint256` arithmetic wraps on overflow; do not assume Solidity-style automatic overflow reverts.

## A loop, one step at a time

`for` has an initialization, a condition, and an update. For `sumBelow(5)`, `i` takes the values `0`, `1`, `2`, `3`, and `4`. The condition `i < limit` is checked before each iteration. Adding them gives `10`. With limit `0`, the body never runs.

Each iteration costs gas. Keep inputs small: a finite loop can still exhaust the runner's gas limit.

## Change it

Before changing any source, use **Try calls** to select `sumBelow(uint256)`, enter `[6]` in **Arguments**, and click **Call function**. Predict the result first. Then add a test comment for this case above `sumBelow` and rerun the tests.

<details>
<summary>Hint and answer</summary>

The extra iteration adds `5`, so the result is **15**. Add `// #[(6) -> 15]` beside the two existing comments. There should now be six passing tests. Do not replace `i < limit` with `i <= limit` unless you also intend to change the function's meaning and its existing expectations.

</details>

**Check your understanding:** why does `adjust(21, false)` return `21` without calling `double`?
