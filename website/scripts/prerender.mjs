// Static HTML is served to people and crawlers alike; there is no user-agent fork.
import { build } from "esbuild";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";

const website = fileURLToPath(new URL("../", import.meta.url));
const dist = resolve(website, "dist");
const cache = resolve(website, "node_modules/.cache/termfetch-prerender");
const evidence = resolve(website, "../qa/website-revision-seo");
await mkdir(cache, { recursive: true });
await mkdir(evidence, { recursive: true });
const template = await readFile(resolve(dist, "index.html"), "utf8");

function replaceSection(html, name, content) {
  const start = `<!--termfetch:${name}:start-->`;
  const end = `<!--termfetch:${name}:end-->`;
  const from = html.indexOf(start);
  const to = html.indexOf(end);
  if (from < 0 || to <= from || html.indexOf(start, from + start.length) >= 0 || html.indexOf(end, to + end.length) >= 0) {
    throw new Error(`Expected one intact ${name} slot in the Vite HTML template`);
  }
  return html.slice(0, from + start.length) + content + html.slice(to);
}

const bundlePath = resolve(cache, "entry-server.mjs");
await build({
  entryPoints: [resolve(website, "src/entry-server.tsx")],
  outfile: bundlePath,
  bundle: true,
  platform: "node",
  target: "node24",
  format: "esm",
  packages: "external",
  jsx: "automatic",
  loader: { ".css": "empty", ".svg": "dataurl", ".png": "dataurl", ".woff2": "dataurl" },
  logLevel: "warning",
});
const { render, ROUTES } = await import(pathToFileURL(bundlePath).href + `?build=${Date.now()}`);
const pages = [];
for (const route of [...ROUTES, { path: "/404", name: "Not found" }]) {
  const output = render(route.path);
  if (!output.html.includes("<main") || !/<h1\b/.test(output.html)) throw new Error(`No meaningful server-rendered content at ${route.path}`);
  const withHead = replaceSection(template, "seo", "\n    " + output.head + "\n    ");
  const html = replaceSection(withHead, "app", `<div id="root">${output.html}</div>`);
  const destination = route.path === "/" ? resolve(dist, "index.html") : route.path === "/404" ? resolve(dist, "404.html") : resolve(dist, "." + route.path, "index.html");
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, html);
  pages.push({ path: route.path, title: output.metadata.title, canonical: output.metadata.canonical, noindex: output.metadata.noindex, htmlBytes: Buffer.byteLength(html) });
}
await writeFile(resolve(evidence, "prerender.json"), JSON.stringify({ status: "Pass", engine: "React static server rendering with hydration", pages }, null, 2) + "\n");
console.log(`Prerendered ${ROUTES.length} indexable routes and a noindex 404 page.`);
