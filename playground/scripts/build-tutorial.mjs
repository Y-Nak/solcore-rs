import { readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, extname, join, relative, resolve, sep } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { checkTutorial } from "./check-tutorial.mjs";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
export const MDBOOK_VERSION = "0.4.52";
const installCommand = `cargo install mdbook --version ${MDBOOK_VERSION} --locked`;

/** Point generated lessons at their sibling playground, including subpath hosts. */
export function rebasePlaygroundLinks(html, htmlFile, publicRoot) {
  const playgroundPath = relative(dirname(htmlFile), publicRoot).split(sep).join("/") || ".";
  return html.replace(
    /\bhref=(['"])https:\/\/playground\.solcore\.soliditylang\.org\/(#\/examples\/[^'"]*)\1/g,
    (_match, quote, route) => `href=${quote}${playgroundPath}/${route}${quote}`,
  );
}

async function rewriteHtml(directory, publicRoot) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      await rewriteHtml(path, publicRoot);
    } else if (entry.isFile() && extname(path) === ".html") {
      const before = await readFile(path, "utf8");
      const after = rebasePlaygroundLinks(before, path, publicRoot);
      if (after !== before) await writeFile(path, after);
    }
  }
}

function runMdbook(args, options) {
  const result = spawnSync("mdbook", args, options);
  if (result.error?.code === "ENOENT") {
    throw new Error(`mdbook was not found. Install the pinned version with: ${installCommand}`);
  }
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const reason = result.signal ? `signal ${result.signal}` : `exit code ${result.status}`;
    throw new Error(`mdbook ${args[0]} failed with ${reason}`);
  }
  return result;
}

export async function buildTutorial(root = repositoryRoot) {
  const errors = await checkTutorial({ root });
  if (errors.length) throw new Error(errors.join("\n"));

  const publicRoot = join(root, "playground/public");
  const destination = join(publicRoot, "tutorial");
  runMdbook(["build", join(root, "solcore-tutorial"), "--dest-dir", destination], { stdio: "inherit" });
  await rewriteHtml(destination, publicRoot);
  console.log(`Tutorial built at ${destination}; example links use the same playground build.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    await buildTutorial();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
