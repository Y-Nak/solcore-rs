"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const { resolveServer } = require("../server-command");

const base = { home: "/home/user", env: {}, platform: "linux", executable: () => false };

test("explicit compiler and legacy server paths keep their arguments separate", () => {
  assert.deepEqual(resolveServer({ ...base, compilerPath: "/My Tools/solcore", serverPath: "/legacy" }),
    { command: "/My Tools/solcore", args: ["lsp"] });
  assert.deepEqual(resolveServer({ ...base, serverPath: "/My Tools/solcore-lsp" }),
    { command: "/My Tools/solcore-lsp", args: [] });
});

test("environment compiler and legacy standalone server are explicit overrides", () => {
  const env = { SOLCORE_PATH: "/new/solcore", SOLCORE_LSP_SERVER: "/old/solcore-lsp" };
  assert.deepEqual(resolveServer({ ...base, env }), { command: "/new/solcore", args: ["lsp"] });
  assert.deepEqual(resolveServer({ ...base, env: { ...env, SOLCORE_PATH: " " } }),
    { command: "/old/solcore-lsp", args: [] });
});

test("Cargo, local bin, and absolute PATH are searched in order", () => {
  const expected = ["/home/user/.cargo/bin/solcore", "/home/user/.local/bin/solcore", "/opt/bin/solcore"];
  const seen = [];
  const result = resolveServer({ ...base, env: { PATH: ":.:relative/bin:/opt/bin" }, executable: p => {
    seen.push(p);
    return p === expected[2];
  } });
  assert.deepEqual(seen, expected);
  assert.deepEqual(result, { command: expected[2], args: ["lsp"] });
});

test("CARGO_HOME is honored and the first installed compiler wins", () => {
  const result = resolveServer({ ...base, env: { CARGO_HOME: "/custom cargo", PATH: "/opt/bin" }, executable: () => true });
  assert.deepEqual(result, { command: "/custom cargo/bin/solcore", args: ["lsp"] });
});

test("Windows looks for .exe with Windows path separators", () => {
  const seen = [];
  const result = resolveServer({ home: "C:\\Users\\Jane", platform: "win32", env: { Path: ".;C:\\Tools" }, executable: p => {
    seen.push(p);
    return p === "C:\\Tools\\solcore.exe";
  } });
  assert.equal(seen.includes("solcore.exe"), false);
  assert.deepEqual(result, { command: "C:\\Tools\\solcore.exe", args: ["lsp"] });
});

test("missing compiler produces installation guidance instead of trying a project binary", () => {
  assert.throws(() => resolveServer(base), /Install the Solcore release/);
});
