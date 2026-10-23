# Local tools and troubleshooting

You can complete the lessons in a browser. Use a local checkout when you want the tutorial, examples, and compiler to come from exactly the same source revision.

## Build and distribute this checkout

Install Rust through rustup, Node.js with npm, `wasm-pack` **0.15.0**, and mdBook (CI pins **0.4.52**). Rustup uses the repository's `rust-toolchain.toml` automatically. From a fresh checkout at the repository root:

```sh
cargo install wasm-pack --version 0.15.0 --locked
cargo install mdbook --version 0.4.52 --locked
cd playground
wasm-pack build ../crates/wasm --target web --out-dir pkg --profile wasm-release
wasm-pack build ../crates/lsp --target web --out-dir pkg --profile wasm-release -- --features wasm
npm ci
cd ../solcore-tutorial
npm run build
npm run preview
```

The build generates `solcore-tutorial/book/`, containing the book, its own project sources in `projects/`, and the bundled editor in `playground/`. Visit `http://127.0.0.1:4177/`, open a chapter, and choose **Try here**. Keep the preview server running while you read.

Copy the entire `book/` output directory to a static web server to distribute it. It contains the editor, compiler, language server, pinned solc build, projects, and supporting assets; it does not depend on a separately deployed Playground. HTTP hosting is required: opening `index.html` through a `file://` URL does not support the module workers. The output also works below a hosting subdirectory when you copy all its files together.

The two initial `wasm-pack` commands generate local packages before dependency installation. The full tutorial build then rebuilds both WASM packages and downloads the pinned solc build if needed. Those development dependencies are needed to build the tutorial, not to read a distributed copy. See also the [playground development guide](https://github.com/Y-Nak/solcore-rs/blob/HEAD/playground/README.md#development) for the shared editor and runtime.

For ongoing book development, run `npm run build:book` from `solcore-tutorial/` after prose, project-source, or frontend changes. It reuses already prepared WASM and solc assets. Run `npm run build` after compiler or language-server changes, and `npm run preview` to serve the result. The ordinary Playground has its own development and build commands; it does not build this tutorial.

## Read the book without building WASM

From the repository root:

```sh
mdbook serve solcore-tutorial --open
```

This is a prose preview with the tutorial's source listings. It does not generate project JSON or bundle the executable editor, so project links are disabled. Use the complete build above for a tutorial you can distribute and run independently. Building requires the repository and shared runtime sources; the generated distribution does not.

## Check a source file locally

Run these commands from the repository root:

```sh
cargo run -p solcore-driver --locked -- --std-root std \
  solcore-tutorial/examples/tutorial-basics/Basics.sol

cargo run -p solcore-driver --locked -- --std-root std --emit-yul \
  solcore-tutorial/examples/tutorial-basics/Basics.sol

cargo run -p solcore-fmt --locked -- --check \
  solcore-tutorial/examples/tutorial-basics/Basics.sol
```

The first command checks the source. The second also emits Yul. Neither command executes the inline test directives. Use the playground's test action to run them. The third checks formatting; use `--write` instead of `--check` when you intend to format the file in place.

## When a result surprises you

| Symptom | What to check |
| --- | --- |
| The inline editor does not appear | Use the complete distribution over HTTP and copy both its `playground/` and `projects/` folders along with the book. Plain `mdbook serve` only previews the prose. |
| A project cannot load | Check that the complete `projects/` folder is present and served with the book. Rebuild the tutorial if you changed its catalog or source files. |
| Reopening an example keeps your edits | This is intentional. Use **Reset project** to restore the original source, after saving anything you want to keep. |
| Opening a full tab does not restore contract state | The views share saved source, not execution sessions. Repeat the manual calls that set up your state. |
| Tests pass but a manual call sees different state | Tests use a separate deployment. Repeat setup through **Try calls** or reset the manual instance. |
| An individual stateful test passes without manual setup | The runner replays preceding `send` steps for that contract. |
| A function is missing from **Try calls** | Check that it is `public`, compilation succeeded, and the contract has no explicit runtime `main`. |
| Source changes make old results look stale | Run again; retained history is not a result for the edited code. |
| A loop halts | Reduce the input. Execution has a gas limit, even for finite loops. |
| Extensions compiles with Sonatina but cannot deploy | Use **solc** for this project. The current Sonatina output exceeds the runner's 1,000,000 gas limit during deployment. |
| Imports or trait methods fail to resolve | Restore the lesson's imports and helper files; adding an arbitrary type annotation does not supply missing trait evidence. |
| Local WASM changes are not reflected in the tutorial | Run `npm run build` from `solcore-tutorial/` and reload the preview. When developing the shared editor directly, use `npm run dev:force` in `playground/` to clear Vite's dependency cache. |

The first `main` example supports only the runner's no-argument, one-word return shape. Public-method examples use the selector dispatcher and the input types supported by the call/test tooling. Dynamic arrays and arbitrary ADTs are not covered by the inline directive value grammar. Compiler support for a type does not automatically mean every external tool supports its ABI representation.

For a reproducible issue, save the exact source and helper files, record the chosen backend, and copy the diagnostic or failing inputs. A project URL alone identifies the bundled source, not your edited program.
