import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import test from "node:test";
import { checkTutorial } from "../scripts/check-tutorial.mjs";

const examples = [{ id: "hello", files: [{ path: "greetings/Hello.sol", content: "" }] }];
const route = "https://playground.solcore.soliditylang.org/#/examples/hello?file=greetings%2FHello.sol&tab=run";

async function fixture(t, chapter) {
  const root = await mkdtemp(join(tmpdir(), "solcore-tutorial-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const source = join(root, "solcore-tutorial/src");
  await mkdir(source, { recursive: true });
  await writeFile(join(source, "SUMMARY.md"), "# Summary\n\n- [Hello](hello.md)\n");
  await writeFile(join(source, "hello.md"), chapter);
  await writeFile(join(root, "Hello.sol"), "function main() returns (word) { return 42; }\n");
  return root;
}

test("tutorial checks source includes and encoded workspace file paths without network access", async (t) => {
  const root = await fixture(t, `# Hello\n\n[Run](${route})\n\n\`\`\`solidity\n{{#include ../../Hello.sol}}\n\`\`\`\n`);
  assert.deepEqual(await checkTutorial({ root, examples }), []);
});

test("tutorial rejects a renamed chapter, deleted source, and copied Solcore snippet", async (t) => {
  const root = await fixture(t, "[Next](renamed.md)\n\n```solidity\n{{#include ../../missing.sol}}\n```\n\n```sol\nfunction old() {}\n```\n");
  await writeFile(join(root, "solcore-tutorial/src/SUMMARY.md"), "- [Missing](deleted.md)\n");
  const errors = await checkTutorial({ root, examples });
  assert.equal(errors.length, 4, errors.join("\n"));
  assert.ok(errors.some((error) => error.includes("deleted.md")));
  assert.ok(errors.some((error) => error.includes("renamed.md")));
  assert.ok(errors.some((error) => error.includes("missing include: ../../missing.sol")));
  assert.ok(errors.some((error) => error.includes("instead of copying code")));
});

test("tutorial rejects unknown examples, stale file selections, and malformed example URLs", async (t) => {
  const root = await fixture(t, [
    `[Unknown](${route.replace("/hello?", "/old-example?")})`,
    `[Wrong file](${route.replace("greetings%2FHello.sol", "Old.sol")})`,
    "[Malformed](https://playground.solcore.soliditylang.org/#/examples/%zz)",
  ].join("\n"));
  const errors = await checkTutorial({ root, examples });
  assert.equal(errors.length, 3, errors.join("\n"));
  assert.ok(errors.some((error) => error.includes("unknown playground example: old-example")));
  assert.ok(errors.some((error) => error.includes("unknown file Old.sol")));
  assert.ok(errors.some((error) => error.includes("malformed playground route")));
});

test("tutorial links accept function signatures and check reference and HTML links", async (t) => {
  const root = await fixture(t, [
    `[Call](${route}&function=answer(uint256))`,
    `[Reference]: ${route.replace("/hello?", "/missing?")}`,
    `<a href="${route.replace("greetings%2FHello.sol", "Missing.sol")}">Example</a>`,
  ].join("\n"));
  const errors = await checkTutorial({ root, examples });
  assert.equal(errors.length, 2, errors.join("\n"));
  assert.ok(errors.some((error) => error.includes("unknown playground example: missing")));
  assert.ok(errors.some((error) => error.includes("unknown file Missing.sol")));
});

test("the shipped tutorial points to the shipped examples and readable chapters", async () => {
  assert.deepEqual(await checkTutorial(), []);
});
