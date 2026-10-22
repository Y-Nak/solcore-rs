# Public functions and tests

**Goal:** call a public function with arguments and turn a result into a repeatable assertion.

[Open in playground: Calculator](https://playground.solcore.soliditylang.org/#/examples/std-usage?tab=run&view=tests)

```solidity
{{#include ../../playground/src/examples/std-usage/Calculator.sol}}
```

## The contract interface

This contract has two public functions and no explicit `main`. The compiler generates a dispatcher: a caller identifies a function and provides encoded arguments. The ABI describes this interface to tools. In the playground, **Try calls** handles the encoding for supported types.

Choose `viaOperator(uint256,uint256)`, enter `[20, 22]` in **Arguments**, and click **Call function**. The decoded result is **42**. A successful **Compile** also makes the **ABI** tab available for inspection.

Arguments use a JSON array, not Solcore constructor syntax. Booleans are `true`/`false`. For integers too large to represent exactly as JavaScript numbers, use quoted decimal strings, for example `["20", "22"]`. Use small inputs for now.

## Assertions belong beside the behavior

`// #[(20, 22) -> 42]` supplies two inputs and the expected result. The comment belongs to the function immediately below it. It is a test directive understood by the playground and test tooling, not an instruction executed by the deployed contract.

Click **Run all tests from start**. All **four** checks should pass: each implementation returns `42` for `(20, 22)` and `7` for `(0, 7)`. **Run test** above an individual comment runs that assertion. Test output shows expected and actual values, so a failed assertion is different from a syntax or type error.

Both implementations add values. `a + b` resolves through library traits; `Num.add(a, b)` calls the method explicitly. You do not need to define a trait yet to use this library behavior.

## Change it

Add the case `(3, 4) -> 7` to both functions. Then deliberately change **only one expectation** to `8`, run that test, and inspect the failure. Correct it afterward.

<details>
<summary>Hint and answer</summary>

Add `// #[(3, 4) -> 7]` above each function, next to its existing directives. With the intentional `8` expectation, compilation should still succeed, but the assertion should report expected `8`, actual `7`. Restoring `7` leaves six passing checks.

</details>

**Check your understanding:** calling the same inputs manually does not check an assertion. Selecting a test can fill the manual controls, but only the test action checks its expected result.
