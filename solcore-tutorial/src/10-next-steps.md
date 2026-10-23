# Next projects and current boundaries

**Goal:** You can now combine types, modules, traits, and storage to read larger
examples. This chapter explores a module that restricts how its state can
be constructed, then suggests a next project.

<div class="tutorial-exercise" data-title="AMM invariants">
<a class="tutorial-example" href="../projects/invariants.json?tab=run&amp;view=tests">Open AMM invariants in a full tab</a>
</div>

```solidity
{{#include ../examples/invariants/Amm.sol}}
```

The representation of `Pool` lives in a separate file:

```solidity
{{#include ../examples/invariants/pool.sol}}
```

`export { Pool, ... }` exports the type without its constructor, unlike
`Pool(*)`. Outside callers create values through `mkPool`, keeping its
initialization check in one place. `swapXforY` returns a new Pool together
with an output amount; the contract stores the updated Pool.
This example records numeric reserves. It does not implement token
transfers, LP share accounting, or a production access-control design.

## Run and observe

Choose **Run → Run tests → Run all tests from start**. All 5 tests pass.
Starting from `(10,1000)`, the committed `swap(10)` leaves `(20,500)`.
Under **Try calls**, use **Reset contract** if shown, select `swap(uint256)`, enter `[10]`,
and press **Call function**; it returns `500`. Call `poolX()` and `poolY()`
without arguments to confirm `20` and `500`.

## Exercise

Read the condition in `mkPool`. If you change the first initial reserve in
Amm's constructor to `uint256(0)`, will compilation or execution fail?
Try it and inspect the result.

<details>
<summary>Hint and solution</summary>

The types still match, so compilation succeeds. Deployment reverts because
`mkPool`'s `require` fails. Calling `poolX()` without arguments in **Try calls**
attempts that initial deployment and reports the failure.
Restore `uint256(10)` and try again. Type constraints and runtime value
checks have distinct roles.

</details>

## Choose a direction

| What to explore | Example | Where to look |
| --- | --- | --- |
| Mappings and absent ownership | [Mini NFT](../projects/mini-nft.json?tab=run&view=calls) | `Option<address>`, `mint`, `exists` |
| Composing behavior with traits | [Composition](../projects/composition.json?tab=run&view=calls) | `engine.sol` and its three contracts |
| Dividing responsibilities across modules | [Extensions](../projects/extensions.json?tab=run&view=calls) | Storage and transfer hooks; use the **solc** backend |
| Lambdas and locking wrappers | [Reentrancy](../projects/reentrancy.json?tab=run&view=calls) | `withWriteLock`, `withReadLock`, `lam` |

Use **solc** for Extensions: its current Sonatina output compiles but exceeds
the browser runner's deployment gas limit.

These are examples for studying design. Composition's signature example
omits replay protection using a nonce, chain ID, and contract address.
The vault examples track credits but omit asset custody. A locking example
alone does not establish the security of a real contract.

Solcore is under development. Do not assume Rust's ownership, borrowing,
and lifetime model, or every existing Solidity feature, applies to it.
Check unfamiliar constructs with the current compiler. Arbitrary ADTs are
not automatically interoperable with existing Solidity ABI tools.
Execution in the browser does not verify deployment on a live network.

For a small next project, add state inspection and transition tests to
Escrow, or implement another ordered type for Registry. Include old
versions and invalid transitions in your tests as well as successful
inputs, so later changes can be checked against those behaviors.

**Check your understanding:** What does hiding the `Pool` constructor prevent, and which value checks are still performed at runtime?
