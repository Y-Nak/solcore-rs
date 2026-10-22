# solcore-tutorial

An English, exercise-led introduction to the current `solcore-rs` implementation. Start with [Welcome](src/introduction.md), or build the mdBook for navigation, search, expandable hints, and complete example listings.

The curriculum moves from a first return value through variables, functions, control flow, tests, storage, enums, modules, traits, generics, and `comptime`, then finishes with a bounded-counter exercise. It assumes basic programming knowledge but no prior Rust or Solidity knowledge.

## Read and build

Keep this directory inside the repository: code listings include the actual tested sources in `playground/src/examples/`.

```sh
cargo install mdbook --version 0.4.52 --locked
mdbook serve solcore-tutorial --open
```

The standalone book uses the public playground, whose deployed revision may differ from this checkout. For a matching local tutorial and playground, first follow the [fresh-checkout setup](src/appendix-tools.md) to generate both WASM packages. After that:

```sh
cd playground
npm ci
npm run dev
```

This also requires the Rust toolchain and wasm-pack 0.15.0; see [local tools](src/appendix-tools.md). Open **Tutorial** in the playground toolbar, or visit `/tutorial/index.html`. `npm run build` builds both together into `playground/dist`, with the book under `tutorial/`. `npm run build:tutorial` alone generates `playground/public/tutorial/`; Vite serves that directory during development and copies it when building. Rebuild the tutorial after editing its text or included examples.

The integrated build rewrites example links to its sibling playground, including deployments under `VITE_BASE`. The Markdown source keeps usable public links. Example links reuse one named browser tab; no compiler iframe is loaded while reading.

## Maintain the learning contract

Each chapter should explain a goal, provide a runnable example, state expected results, and include an exercise with a hint or answer. Add runnable code to the playground registry and tests, then use a fenced mdBook `include` in the chapter. Do not maintain a second copy of complete code in prose. Update expectations when behavior changes.

```sh
cargo test -p solcore-driver --test playground_examples --locked
cd playground
npm run test:tutorial
npm run build:tutorial
npm run test:wasm
```

The native check compiles all playground sources. The tutorial check validates chapters, includes, example IDs, file selections, and integration URLs without network requests. The WASM tests execute the learning examples on both solc and Sonatina; rebuild WASM first after compiler changes. These checks are part of the existing CI workflow.

The initial material was developed against base commit `d796716186242e3d53a50dd3c096a8077af3ac88`. Thereafter, included sources and CI keep it aligned with its checkout. The standard-library revision and intentional compatibility differences remain documented in the repository's [README](../README.md) and [semantic differences](../SEMANTIC_DIFFERENCES.md).

See [the design notes](src/appendix-design.md) for the official Rust, Go, and Svelte tutorials that informed the teaching format and the playground integration decision. This book follows the repository's [Apache-2.0 license](../LICENSE).
