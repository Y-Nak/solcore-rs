import assert from "node:assert/strict";
import { join } from "node:path";
import test from "node:test";
import { rebasePlaygroundLinks } from "../scripts/build-tutorial.mjs";

const publicRoot = join(process.cwd(), "public");
const route = "#/examples/option?file=option.sol&amp;tab=run&amp;view=tests";
const link = `<a href="https://playground.solcore.soliditylang.org/${route}">Try it</a>`;

test("integrated tutorial links select the sibling playground from a root chapter", () => {
  const page = join(publicRoot, "tutorial/06-modules.html");
  assert.equal(rebasePlaygroundLinks(link, page, publicRoot), `<a href="../${route}">Try it</a>`);
});

test("nested chapters and print output keep working under a deployment subpath", () => {
  for (const [chapter, expected] of [
    ["print.html", "../"],
    ["advanced/modules.html", "../../"],
    ["advanced/traits/generics.html", "../../../"],
  ]) {
    const page = join(publicRoot, "tutorial", chapter);
    const html = rebasePlaygroundLinks(link, page, publicRoot);
    const href = html.match(/href="([^"]+)"/)[1].replaceAll("&amp;", "&");
    assert.ok(href.startsWith(expected));
    const resolved = new URL(href, `https://example.org/repo/tutorial/${chapter}`);
    assert.equal(resolved.href, `https://example.org/repo/${route.replaceAll("&amp;", "&")}`);
  }
});

test("rewriting preserves unrelated links, source text, and already relative links", () => {
  const html = [
    '<a href="https://doc.rust-lang.org/book/">Rust Book</a>',
    '<a href="https://playground.solcore.soliditylang.org/">Playground home</a>',
    '<a href="https://example.org/#/examples/option">Other playground</a>',
    '<a href="../#/examples/option">Already local</a>',
    '<a href="07-traits.html">Next</a>',
    '<code>https://playground.solcore.soliditylang.org/#/examples/option</code>',
  ].join("\n");
  assert.equal(rebasePlaygroundLinks(html, join(publicRoot, "tutorial/index.html"), publicRoot), html);
});

test("rewriting handles every generated href and preserves quote style and encoded signatures", () => {
  const single = "<a href='https://playground.solcore.soliditylang.org/#/examples/trait?function=toggle%28%29'>Call</a>";
  const html = rebasePlaygroundLinks(`${link}\n${single}`, join(publicRoot, "tutorial/index.html"), publicRoot);
  assert.equal(html, `<a href="../${route}">Try it</a>\n<a href='../#/examples/trait?function=toggle%28%29'>Call</a>`);
});
