# solcore-tutorial

An English, exercise-led introduction to the current `solcore-rs` implementation. Start with [Welcome](src/introduction.md), or build the mdBook for navigation, search, expandable hints, and complete example listings.

The curriculum moves from a first return value through variables, functions, control flow, tests, storage, enums, modules, traits, generics, and `comptime`, then finishes with a bounded-counter exercise. It assumes basic programming knowledge but no prior Rust or Solidity knowledge.

## Build and distribute the standalone tutorial

The distribution includes the book, editor, compiler, language server, solc, browser EVM, and tutorial project sources. Readers can edit and run code inside a lesson without using the public Playground or installing compiler tools. **Try here** loads the bundled editor in an iframe on demand. **Open in full tab** provides more room and keeps the same project's saved edits. Only one exercise is open in a chapter at a time.

The tutorial owns its [project catalog](examples/catalog.json), `.sol` examples, chapter source, theme, and build commands. It reuses the Playground application as a project editor; tutorial exercises are not registered in the ordinary Playground example menu. The Playground does not build or distribute this book.

Install Rust through rustup, Node.js with npm, wasm-pack 0.15.0, and mdBook 0.4.52. Prepare the shared runtime dependencies from a fresh repository checkout:

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

Open `http://127.0.0.1:4177/`. Copy or archive the **entire `solcore-tutorial/book/` directory** to distribute it; the repository and a separate Playground deployment are not needed by readers. The directory contains `projects/` with the generated tutorial projects and `playground/` with the editor and runtime assets.

Serve it over HTTP(S), with `.wasm` files served as `application/wasm`. Opening HTML directly with `file://` does not support the module workers. Relative asset URLs allow the same directory to be hosted at `/` or a nested path without rebuilding. Building requires dependency downloads; reading, editing, compilation, and execution use the bundled files at runtime. External reference links remain optional.

Each project has its own local saved workspace, separate from the ordinary Playground. Chapters using the same project share those edits. **Reset project** restores its original source. Saves are local to the browser and origin; URLs do not contain edited code.

## Develop the book

Run these commands from `solcore-tutorial/`:

```sh
npm run build       # Rebuild shared WASM and solc assets, then the tutorial.
npm run build:book  # Reuse prepared WASM/solc assets; rebuild prose, projects, and editor.
npm run preview    # Serve book/ at http://127.0.0.1:4177/.
```

Use the full build after compiler or language-server changes. The faster book build is useful after editing prose, curriculum sources, or editor frontend code. Both produce a standalone `book/`; neither changes the ordinary Playground's example list or output directory.

For a prose-only preview, `mdbook serve solcore-tutorial --open` works from the repository root. This includes the tutorial's source listings but does not generate project JSON or bundle the editor. Project links are disabled in that preview. Re-run the complete tutorial build before distribution.

## Maintain the learning contract

Each chapter should explain a goal, provide a runnable example, state expected results, and include an exercise with a hint or answer. Add a project to `examples/catalog.json` and its source files to `examples/<id>/`. Each catalog entry declares `id`, `name`, `description`, `entry`, and a list of workspace-relative `files`. Include every helper source needed by that project. Keep complete code in those files and use fenced mdBook includes in chapters; do not duplicate complete code in prose.

Chapter links use `../projects/<id>.json`, optionally followed by a query selecting a file, tab, or view. The tutorial build generates the project and rewrites the link to the bundled editor. Mark a primary exercise link with `class="tutorial-example"` inside a `class="tutorial-exercise"` container with a descriptive `data-title`. The theme enhances that link into an accessible inline editor in complete builds. Other project links remain full-tab links. Update expectations when behavior changes.

From `solcore-tutorial/`, run the checks for the curriculum and distribution:

```sh
npm test
npm run test:wasm
npm run test:bundle
```

Build first before checking the distribution, and prepare current WASM assets before running execution tests. Tutorial checks validate chapters, source includes, catalog entries, file selections, and generated project links. Runtime tests execute curriculum sources using the shared compiler and browser EVM. Distribution checks verify local HTML links and bundled compiler, language-server, and solc assets. These checks keep the book and its own examples aligned while allowing the ordinary Playground examples to evolve independently.

The Rust compiler and its regression tests define the supported language; see the repository's [README](../README.md) and [standard-library maintenance policy](../std/README.md).

See [the design notes](src/appendix-design.md) for the official Rust, Go, and Svelte tutorials that informed the teaching format and integration decision. This book follows the repository's [Apache-2.0 license](../LICENSE).
