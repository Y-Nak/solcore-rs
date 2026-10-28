"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

function isExecutable(file) {
  try {
    fs.accessSync(file, process.platform === "win32" ? fs.constants.F_OK : fs.constants.X_OK);
    return fs.statSync(file).isFile();
  } catch {
    return false;
  }
}

// Resolve on the extension host (also correct for SSH, WSL, and containers).
// Explicit overrides are authoritative: a broken override must not silently
// select a different compiler. Commands and arguments are never shell strings.
function resolveServer({
  compilerPath = "",
  serverPath = "",
  env = process.env,
  home = os.homedir(),
  platform = process.platform,
  executable = isExecutable,
} = {}) {
  const compiler = compilerPath.trim();
  const legacy = serverPath.trim();
  if (compiler) return { command: compiler, args: ["lsp"] };
  if (legacy) return { command: legacy, args: [] };
  if (env.SOLCORE_PATH?.trim()) return { command: env.SOLCORE_PATH.trim(), args: ["lsp"] };
  if (env.SOLCORE_LSP_SERVER?.trim()) return { command: env.SOLCORE_LSP_SERVER.trim(), args: [] };

  const paths = platform === "win32" ? path.win32 : path.posix;
  const name = platform === "win32" ? "solcore.exe" : "solcore";
  const cargoHome = env.CARGO_HOME || paths.join(home, ".cargo");
  const directories = [
    paths.join(cargoHome, "bin"),
    paths.join(home, ".local", "bin"),
    ...(env.PATH || env.Path || "").split(paths.delimiter),
  ];
  for (const directory of directories) {
    // Empty and relative PATH entries can select a binary in the project.
    if (!paths.isAbsolute(directory)) continue;
    const candidate = paths.join(directory, name);
    if (executable(candidate)) return { command: candidate, args: ["lsp"] };
  }
  throw new Error("Install the Solcore release and set solcore.path or SOLCORE_PATH to its solcore executable");
}

module.exports = { resolveServer };
