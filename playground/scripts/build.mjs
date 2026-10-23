import { cp } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { build } from "vite";

const playgroundRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

/** Build a portable editor/runtime bundle for standalone use or embedding. */
export async function buildPlayground({ outDir = join(playgroundRoot, "dist"), base } = {}) {
  await build({
    configFile: join(playgroundRoot, "vite.config.ts"),
    root: playgroundRoot,
    ...(base === undefined ? {} : { base }),
    publicDir: false,
    build: { outDir, emptyOutDir: true },
  });
  // WASM and workers are emitted by Vite. solc is the only unbundled runtime.
  await cp(join(playgroundRoot, "public/solc"), join(outDir, "solc"), { recursive: true });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  await buildPlayground();
}
