import { readFile, readdir, stat } from "node:fs/promises";
import { dirname, extname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { build } from "esbuild";
import { readExampleLocation } from "../src/share/exampleLink.js";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const playgroundOrigin = "https://playground.solcore.soliditylang.org";

/** Load the same registry and source files that Vite ships, without a browser. */
export async function loadExamples(root = repositoryRoot) {
  const bundled = await build({
    entryPoints: [join(root, "playground/src/examples/index.ts")],
    bundle: true,
    write: false,
    platform: "node",
    format: "esm",
    plugins: [{
      name: "tutorial-example-sources",
      setup(builder) {
        builder.onResolve({ filter: /\.sol\?raw$/ }, (args) => ({
          path: resolve(args.resolveDir, args.path.slice(0, -4)),
          namespace: "solcore-source",
        }));
        builder.onLoad({ filter: /.*/, namespace: "solcore-source" }, async (args) => ({
          contents: await readFile(args.path, "utf8"),
          loader: "text",
        }));
      },
    }],
  });
  const module = await import(`data:text/javascript;base64,${Buffer.from(bundled.outputFiles[0].text).toString("base64")}`);
  return module.examples;
}

async function markdownFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await markdownFiles(path));
    else if (entry.isFile() && extname(path) === ".md") files.push(path);
  }
  return files.sort();
}

async function isFile(path) {
  try { return (await stat(path)).isFile(); }
  catch (error) {
    if (error.code === "ENOENT" || error.code === "ENOTDIR") return false;
    throw error;
  }
}

// Handle balanced parentheses in ordinary inline Markdown URLs, including
// function signatures. Reference links and HTML hrefs are checked as well.
function links(markdown) {
  const found = [];
  for (const match of markdown.matchAll(/!?\[[^\]\n]*\]\(/g)) {
    const start = match.index + match[0].length;
    if (markdown[start] === "<") {
      const end = markdown.indexOf(">", start);
      if (end !== -1) found.push(markdown.slice(start + 1, end));
      continue;
    }
    let depth = 1;
    let end = start;
    for (; end < markdown.length; end += 1) {
      const character = markdown[end];
      if (character === "\\") { end += 1; continue; }
      if (character === "(") depth += 1;
      if (character === ")" && --depth === 0) break;
      if (/\s/.test(character)) break;
    }
    found.push(markdown.slice(start, end));
  }
  for (const match of markdown.matchAll(/^\s*\[[^\]]+\]:\s*<?([^\s>]+)>?/gm)) found.push(match[1]);
  for (const match of markdown.matchAll(/\bhref=["']([^"']+)["']/g)) found.push(match[1].replaceAll("&amp;", "&"));
  return found;
}

/** Validate local book dependencies and routes; never request external URLs. */
export async function checkTutorial({ root = repositoryRoot, examples } = {}) {
  const source = join(root, "solcore-tutorial/src");
  const registry = examples ?? await loadExamples(root);
  const examplesById = new Map(registry.map((example) => [example.id, example]));
  const errors = [];
  const report = (file, message) => errors.push(`${relative(root, file)}: ${message}`);
  const summary = join(source, "SUMMARY.md");
  if (!await isFile(summary)) return [`${relative(root, summary)}: missing table of contents`];

  for (const file of await markdownFiles(source)) {
    const markdown = await readFile(file, "utf8");
    for (const match of markdown.matchAll(/\{\{#include\s+([^}\n]+)\}\}/g)) {
      const target = match[1].trim().split(":")[0];
      if (!await isFile(resolve(dirname(file), target))) report(file, `missing include: ${target}`);
    }
    for (const match of markdown.matchAll(/^```(?:sol|solcore|solidity)\s*\n([\s\S]*?)^```\s*$/gm)) {
      if (!/^\s*\{\{#include\s+[^}\n]+\}\}\s*$/.test(match[1])) {
        report(file, "Solcore code blocks must include a tested source file instead of copying code");
      }
    }

    // Code samples are not links. Includes above are checked before removing
    // fenced examples so documentation about URL syntax can use text fences.
    const prose = markdown.replace(/^```[^\n]*\n[\s\S]*?^```\s*$/gm, "");
    for (const target of links(prose)) {
      if (!target || target.startsWith("#")) continue;
      if (/^[a-z][a-z\d+.-]*:/i.test(target) || target.startsWith("//")) {
        let url;
        try { url = new URL(target); }
        catch { report(file, `malformed URL: ${target}`); continue; }
        if (url.origin !== playgroundOrigin || !url.hash.startsWith("#/examples")) continue;
        const location = readExampleLocation(url.hash);
        if (!location) { report(file, `malformed playground route: ${target}`); continue; }
        const example = examplesById.get(location.id);
        if (!example) { report(file, `unknown playground example: ${location.id}`); continue; }
        if (location.view.file && !example.files.some((item) => item.path === location.view.file)) {
          report(file, `unknown file ${location.view.file} in playground example ${location.id}`);
        }
      } else {
        const path = target.split(/[?#]/)[0];
        let decoded;
        try { decoded = decodeURIComponent(path); }
        catch { report(file, `malformed local link: ${target}`); continue; }
        if (!await isFile(resolve(dirname(file), decoded))) report(file, `missing local link: ${target}`);
      }
    }
  }
  return errors;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const errors = await checkTutorial();
  if (errors.length) {
    console.error(errors.join("\n"));
    process.exitCode = 1;
  } else {
    console.log("Tutorial chapter links, source includes, and playground routes are valid.");
  }
}
