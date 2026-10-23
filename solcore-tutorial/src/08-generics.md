# Generics and constraints

**Goal:** In this chapter, express a reusable operation with a type parameter, then
state its required capabilities with `where`. The example is a version
registry, with an ordering that compares the major version first.

<div class="tutorial-exercise" data-title="Generic Registry">
<a class="tutorial-example" href="../projects/generics.json?tab=run&amp;view=tests">Open Generic Registry in a full tab</a>
</div>

```solidity
{{#include ../examples/generics/Registry.sol}}
```

`max<a>(x: a, y: a)` takes two values of the same type and returns that
type. Because it uses `>`, it requires `where a: Ord`. The function itself
does not inspect any `Version` fields.

`Version` is an enum carrying two integers, extracted with `match`.
`impl Eq<Version>` defines equality; `impl Ord<Version>` defines ordering.
`Ord` requires `Eq`, so the example provides both implementations.
Different major versions are compared by major; minor versions matter only
when the majors are equal. Thus `2.1` is newer than `1.99`.

`publish` stores the `max` of the current and incoming version. This
reuses the ordering definition whenever a new value arrives.

## Run and observe

Choose **Run → Run tests → Run all tests from start**. All 6 tests pass.
Publishing `1.9 → 2.1 → 1.99 → 2.0` leaves the registry at `2.1`.

Open **Try calls**, use **Reset contract** if shown, then select these functions and
press **Call function** in order:

| Function | Arguments | Result |
| --- | --- | --- |
| `publish(uint256,uint256)` | `[1,9]` | Success |
| `publish(uint256,uint256)` | `[2,1]` | Success |
| `publish(uint256,uint256)` | `[1,99]` | Success |
| `newestMajor()` | `[]` | `2` |
| `newestMinor()` | `[]` | `1` |

The `[]` rows are no-argument functions: no Arguments field appears for them.

## Exercise

Append `// #[send(2, 7)]` to the send comments immediately above
`publish`. Decide how the two final expected-value tests should change
before running them.

<details>
<summary>Hint and solution</summary>

The major stays `2`, and the minor becomes `7`. Change only the
`newestMinor` assertion to `// #[() -> 7]`; all 7 tests pass.
Appending another send, `// #[send(2, 3)]`, still leaves the result at
`2.7`. This checks that the registry also refuses to move backwards within
a single major version.

</details>

**Check your understanding:** What operation in `max` makes the `Ord` constraint necessary?
