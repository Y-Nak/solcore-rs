# Computing values at compile time

**Goal:** In this chapter, use `comptime` to calculate a value before execution.
Distinguish values known while compiling from arguments supplied by a
future caller.

[Open Answer in the Playground](https://playground.solcore.soliditylang.org/#/examples/comptime?tab=run&view=tests)

```solidity
{{#include ../../playground/src/examples/comptime/Answer.sol}}
```

The parameter `comptime value: uint256` requires a value known at compile
time. The result type `comptime<uint256>` also describes a compile-time
value. Here `uint256(21)` is known, so specialization can evaluate
`21 + 21`. The `return result` in `answer()` uses that result as an ordinary
`uint256` return value.

This does not store the result of a previous execution in contract storage.
A value that changes with each external call cannot be passed as a
compile-time argument. An ordinary parameter of a public function, for
example, is supplied by a future caller and is not known while compiling.
Even if frontend checking accepts a program, the full compilation can fail
during specialization when a required compile-time value is unavailable.

## Run and observe

Choose **Run → Run tests → Run all tests from start**. The one test passes,
with expected and actual values both `42`.
Under **Try calls**, choose `answer()` and press **Call function**.
It takes no arguments, so there is no Arguments field. Its result is also `42`.

After **Compile**, inspect **Hull** or **Yul** to observe generated code.
The exact intermediate representation can change with the backend and
optimization. Use the returned value as this chapter's check, rather than
relying on an exact output string in those tabs.

## Exercise

Change `double(uint256(21))` to use `22`. Run the test once without changing
its assertion, then update the expectation to match the intended result.

<details>
<summary>Hint and solution</summary>

The first test run fails: it expects `42` but receives `44`. Distinguish
this from a compilation error. A program can compile and execute while
failing its expected-value test.

Change the comment to `// #[() -> 44]` and choose
**Run all tests from start**; it passes.
Changing the body of `double` to `value + value + value` would produce
`66` instead. The compile-time calculation and the execution-time test of
its result can be checked separately.

</details>

**Check your understanding:** Why is a future public-function argument unavailable to a `comptime` parameter?
