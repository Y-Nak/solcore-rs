# Defining behavior with traits

**Goal:** In this chapter, declare an operation in a `trait` and provide its
implementation with `impl`. An enum represents a switch's state; a trait
defines how to change that state.

<div class="tutorial-exercise" data-title="LightSwitch traits">
<a class="tutorial-example" href="../projects/trait.json?tab=run&amp;view=tests">Open LightSwitch traits in a full tab</a>
</div>

```solidity
{{#include ../examples/trait/LightSwitch.sol}}
```

`trait Toggle<a>` requires a `toggle` function that takes an `a` and returns
an `a`. `impl Toggle<Switch>` supplies the implementation for `Switch`:
`Switch.Off` becomes `Switch.On`, and `Switch.On` becomes `Switch.Off`.
These are Solcore's current `trait Toggle<a>` and `impl Toggle<Switch>`
forms; their syntax is not interchangeable with Rust's.

The contract's `flip` method calls `Toggle.toggle(state)` and stores the
returned value. The implementation is selected using the type of `state`.
A trait defines operations for a type; it does not establish an inheritance
relationship between contracts.

## Run and observe

Choose **Run → Run tests → Run all tests from start**. Both tests pass.
The `#[send()]` on `flip` runs first, so `isOn` then returns `true`.
Running the individual **Run test** for `isOn` also replays its preceding
send.

For a manual experiment, open **Try calls**, use **Reset contract** if shown, then
select each function below and press **Call function**. These functions take no arguments, so there is no Arguments field to fill in.

| Function | Result |
| --- | --- |
| `isOn()` | `false` |
| `flip()` | Success |
| `isOn()` | `true` |
| `flip()` | Success |
| `isOn()` | `false` |

Tests and manual calls use independent state. A test returning `true` does
not change a newly deployed manual contract: its constructor still starts
with `Off`.

## Exercise

Add a `flipTwice()` method that returns the switch to its previous state.
Call `Toggle.toggle` twice; the intermediate value need not be stored.
Check the result through Try calls.

<details>
<summary>Hint and solution</summary>

Inside `LightSwitch`, add a public `flipTwice` function with no parameters
and no return value, using the declaration of `flip` as a guide.
Its only statement is `state = Toggle.toggle(Toggle.toggle(state));`.

After editing, call `isOn()`, `flipTwice()`, then `isOn()`, all without arguments.
Both readings are `false`. Next call `flip()`, `flipTwice()`, then `isOn()`;
this reading is `true`. Testing both starting states shows that the method
preserves its input state instead of always resetting it to `Off`.

</details>

**Check your understanding:** which declaration specifies the required operation, and which one defines its behavior for `Switch`?
