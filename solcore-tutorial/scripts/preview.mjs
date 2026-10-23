import { createReadStream } from "node:fs";
import { realpath, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = await realpath(fileURLToPath(new URL("../book", import.meta.url))).catch(() => {
  throw new Error("Build the tutorial with npm run build:book before starting the preview.");
});
const port = Number(process.env.PORT ?? 4177);
const types = {
  ".html": "text/html; charset=utf-8", ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8", ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm", ".svg": "image/svg+xml", ".png": "image/png",
  ".woff": "font/woff", ".woff2": "font/woff2", ".ttf": "font/ttf",
};
function inside(path) {
  const rel = relative(root, path);
  return rel !== ".." && !rel.startsWith(`..${sep}`);
}
createServer(async (request, response) => {
  if (!["GET", "HEAD"].includes(request.method)) {
    response.writeHead(405, { Allow: "GET, HEAD" }).end();
    return;
  }
  try {
    const pathname = decodeURIComponent(new URL(request.url, "http://preview.invalid").pathname);
    let path = resolve(root, `.${pathname}`);
    if (!inside(path)) throw new Error("Outside book");
    let info = await stat(path);
    if (info.isDirectory()) {
      path = join(path, "index.html");
      info = await stat(path);
    }
    path = await realpath(path);
    if (!inside(path) || !info.isFile()) throw new Error("Outside book");
    response.writeHead(200, { "Content-Type": types[extname(path)] ?? "application/octet-stream",
      "Content-Length": info.size, "Cache-Control": "no-cache" });
    if (request.method === "HEAD") response.end();
    else createReadStream(path).on("error", () => response.destroy()).pipe(response);
  } catch {
    response.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" }).end("Not found");
  }
}).listen(port, "127.0.0.1", () => {
  console.log(`Tutorial preview: http://127.0.0.1:${port}/`);
});
