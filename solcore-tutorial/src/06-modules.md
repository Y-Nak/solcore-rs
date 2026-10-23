# Modules and Option

**Goal:** In this chapter, split code across files and use `export` to choose which
names other modules can use. Represent a missing result explicitly, so a
caller can decide how to handle it.
The `Option` used here is defined in the example's `option.sol`; it is not
a built-in standard-library type.

<div class="tutorial-exercise" data-title="Splitter and Option">
<a class="tutorial-example" href="../projects/option.json?tab=run&amp;view=tests">Open Splitter and Option in a full tab</a>
</div>

## A reusable module

Select `option.sol` in the file list.

```solidity
{{#include ../examples/option/option.sol}}
```

The `a` in `Option<a>` is a type parameter. For `Option<uint256>`, `Some`
carries an integer and `None` carries no value. `unwrapOr<a>` is a generic
function: its control flow works for different choices of `a`.

`checkedDiv` explicitly checks whether the divisor is zero and returns
`None` in that case. Do not infer that every `/` operation automatically
reverts on division by zero. This function's guarantee is that it avoids
performing the division and reports the missing result to its caller.

`export { Option(*), ... }` exports the type and its constructors.
The `where a: Eq` constraint on `contains` requires a type that supports
equality; the next chapters explain traits and constraints.

## Using the module

Return to `Splitter.sol`. `from option` refers to the example's
`option.sol`. Selecting a file to read does not require changing the entry
file used for compilation.

```solidity
{{#include ../examples/option/Splitter.sol}}
```

`share` uses `unwrapOr` to choose `0` when the result is absent.
`remainder` uses `match`; if there are no recipients, it returns the
original pot. The same `Option` value lets each caller choose a policy
appropriate to its purpose.

## Run and observe

Choose **Run → Run tests → Run all tests from start**. All 4 tests pass.
You can also select each function under **Try calls**, enter its Arguments,
and press **Call function**:

| Function | Arguments | Result |
| --- | --- | --- |
| `share(uint256,uint256)` | `[10,3]` | `3` |
| `remainder(uint256,uint256)` | `[10,3]` | `1` |
| `share(uint256,uint256)` | `[10,0]` | `0` |
| `remainder(uint256,uint256)` | `[10,0]` | `10` |

## Exercise

Add a test for a pot of `14` and `4` recipients to each function. Predict
both the share and the remainder before running the tests.

<details>
<summary>Hint and solution</summary>

Add `// #[(14, 4) -> 3]` immediately above `share`, and
`// #[(14, 4) -> 2]` immediately above `remainder`.
Integer division gives a share of `3`; the remainder is `14 - 3 * 4 = 2`.
**Run all tests from start** now passes 6 tests.

</details>

**Check your understanding:** Why can `share` and `remainder` choose different behavior when `checkedDiv` returns `None`?
