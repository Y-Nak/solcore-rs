# Local tools and troubleshooting

You can complete the lessons in a browser. Use a local checkout when you want the tutorial, examples, and compiler to come from exactly the same source revision.

## Run this checkout

Install Rust through rustup, Node.js with npm, `wasm-pack` **0.15.0**, and mdBook (CI pins **0.4.52**). Rustup uses the repository's `rust-toolchain.toml` automatically. From a fresh checkout at the repository root:

```sh
cargo install wasm-pack --version 0.15.0 --locked
cargo install mdbook --version 0.4.52 --locked
cd playground
wasm-pack build ../crates/wasm --target web --out-dir pkg --profile wasm-release
wasm-pack build ../crates/lsp --target web --out-dir pkg --profile wasm-release -- --features wasm
npm ci
npm run dev
```

The development command builds both local WASM packages and the tutorial, and downloads the pinned solc build if needed. It prints the local server address. Open that address and choose **Tutorial**, or append `/tutorial/index.html` to it. Keep the development server running while you work.

The two initial `wasm-pack` commands generate the local packages before dependency installation. On subsequent runs, `npm run dev` handles rebuilding them. JavaScript dependencies include the pinned WASM optimizer used by the normal build scripts. See also the [playground development guide](https://github.com/Y-Nak/solcore-rs/blob/HEAD/playground/README.md#development).

To build the complete static site, run `npm run build` from `playground/`. Run `npm run preview` to serve it locally. The tutorial is included at `/tutorial/index.html`; the same setup also works below a deployment subpath configured with `VITE_BASE`.

## Read the book without building WASM

From the repository root:

```sh
mdbook serve solcore-tutorial --open
```

This serves the text and included example sources. Its example links point to the public playground, which might not yet include additions from this checkout. For an integrated book with matching local examples, use the development workflow above. The book depends on sibling playground sources, so keep it inside this repository.

## Check a source file locally

Run these commands from the repository root:

```sh
cargo run -p solcore-driver --locked -- --std-root std \
  playground/src/examples/tutorial-basics/Basics.sol

cargo run -p solcore-driver --locked -- --std-root std --emit-yul \
  playground/src/examples/tutorial-basics/Basics.sol

cargo run -p solcore-fmt --locked -- --check \
  playground/src/examples/tutorial-basics/Basics.sol
```

The first command checks the source. The second also emits Yul. Neither command executes the inline test directives. Use the playground's test action to run them. The third checks formatting; use `--write` instead of `--check` when you intend to format the file in place.

## When a result surprises you

| Symptom | What to check |
| --- | --- |
| A linked example is missing | Use this checkout's local build; the public deployment may be older. |
| Reopening an example keeps your edits | **Reset workspace** returns to Hello; then reopen the lesson link. Save your work first. |
| Tests pass but a manual call sees different state | Tests use a separate deployment. Repeat setup through **Try calls** or reset the manual instance. |
| An individual stateful test passes without manual setup | The runner replays preceding `send` steps for that contract. |
| A function is missing from **Try calls** | Check that it is `public`, compilation succeeded, and the contract has no explicit runtime `main`. |
| Source changes make old results look stale | Run again; retained history is not a result for the edited code. |
| A loop halts | Reduce the input. Execution has a gas limit, even for finite loops. |
| Imports or trait methods fail to resolve | Restore the lesson's imports and helper files; adding an arbitrary type annotation does not supply missing trait evidence. |
| Local WASM changes are not reflected in the UI | Restart with `npm run dev:force` to clear Vite's dependency cache. |

The first `main` example supports only the runner's no-argument, one-word return shape. Public-method examples use the selector dispatcher and the input types supported by the call/test tooling. Dynamic arrays and arbitrary ADTs are not covered by the inline directive value grammar. Compiler support for a type does not automatically mean every external tool supports its ABI representation.

For a reproducible issue, save the exact source and helper files, record the chosen backend, and copy the diagnostic or failing inputs. A playground URL alone identifies the bundled example, not your edited program.
