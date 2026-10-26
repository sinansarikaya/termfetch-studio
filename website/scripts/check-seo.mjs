import assert from "node:assert/strict";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const website = fileURLToPath(new URL("../", import.meta.url));
const dist = resolve(website, "dist");
const evidence = resolve(website, "../qa/website-revision-seo");
await mkdir(evidence, { recursive: true });
const baseAt = process.argv.indexOf("--base");
const base = baseAt >= 0 ? process.argv[baseAt + 1]?.replace(/\/$/, "") : null;
if (baseAt >= 0 && !base) throw new Error("--base requires the site origin");
const paths = ["/", "/themes", "/presets", "/downloads", "/docs", "/docs/getting-started", "/releases", "/faq"];
const origin = "https://termfetch-studio.sinansarikaya.dev";
const checks = [];
const entities = (value) => value.replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&#(?:x([0-9a-f]+)|(\d+));/gi, (_, hex, dec) => String.fromCodePoint(parseInt(hex || dec, hex ? 16 : 10)));
const plain = (html) => entities(html.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, " ").replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, " ").replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim());
const attributes = (source) => Object.fromEntries([...source.matchAll(/([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/g)].map((match) => [match[1].toLowerCase(), entities(match[2] ?? match[3])]));
const tags = (html, name) => [...html.matchAll(new RegExp(`<${name}\\b([^>]*)>`, "gi"))].map((match) => attributes(match[1]));
const report = { status: "Fail", target: base || "built static HTML", checkedAt: new Date().toISOString(), checks, limitations: ["These are technical/content checks, not a ranking, indexing or AI-citation guarantee.", "Browser hydration, visual fidelity, keyboard and responsive checks are covered separately by the headed browser regression."] };

async function get(path, expected = 200) {
  if (base) {
    const response = await fetch(base + path, { redirect: "manual", signal: AbortSignal.timeout(20000) });
    assert.equal(response.status, expected, `${path}: direct response must be ${expected}, without a redirect`);
    return response.text();
  }
  const filename = path === "/" ? "index.html" : path.endsWith(".txt") || path.endsWith(".xml") || path.endsWith(".html") ? path.slice(1) : path.slice(1) + "/index.html";
  return readFile(resolve(dist, filename), "utf8");
}

try {
  const sitemap = await get("/sitemap.xml");
  const locations = [...sitemap.matchAll(/<loc>([^<]+)<\/loc>/g)].map((match) => entities(match[1]));
  assert.deepEqual([...locations].sort(), paths.map((path) => origin + path).sort(), "Sitemap has exactly the indexable canonical routes");
  const robots = await get("/robots.txt");
  assert.match(robots, /User-agent:\s*\*/i);
  assert.match(robots, /Allow:\s*\//i);
  assert.ok(!/Disallow:\s*\/\s*(?:\r?\n|$)/i.test(robots), "Robots must not block the entire site");
  assert.ok(robots.includes(origin + "/sitemap.xml"));
  const llms = await get("/llms.txt");
  assert.ok(llms.includes(origin + "/docs") && llms.includes("pending"));
  checks.push("Robots, sitemap and optional llms.txt describe the actual public origin and route set");

  const titles = new Set();
  const descriptions = new Set();
  const documents = new Map();
  for (const path of paths) documents.set(path, await get(path));
  const resolveRoute = (value, pathname) => {
    const url = new URL(value, origin + pathname);
    return { url, path: url.pathname === "/" ? "/" : url.pathname.replace(/\/+$/, "") };
  };
  for (const [path, html] of documents) {
    const titleMatches = [...html.matchAll(/<title\b[^>]*>([\s\S]*?)<\/title>/gi)];
    assert.equal(titleMatches.length, 1, `${path}: one title`);
    const title = plain(titleMatches[0][1]);
    assert.ok(title.length > 15 && title.length < 110, `${path}: useful descriptive title`);
    assert.ok(!titles.has(title), `${path}: unique page title`); titles.add(title);
    const meta = tags(html, "meta");
    const description = meta.filter((tag) => tag.name === "description");
    assert.equal(description.length, 1, `${path}: one description`);
    assert.ok(description[0].content.length >= 60 && description[0].content.length <= 220);
    assert.ok(!descriptions.has(description[0].content), `${path}: unique description`); descriptions.add(description[0].content);
    assert.equal(tags(html, "link").filter((tag) => tag.rel === "canonical").length, 1);
    assert.equal(tags(html, "link").find((tag) => tag.rel === "canonical")?.href, origin + path);
    assert.equal(meta.find((tag) => tag.property === "og:url")?.content, origin + path);
    assert.equal(meta.find((tag) => tag.property === "og:title")?.content, title);
    assert.equal(meta.find((tag) => tag.name === "twitter:title")?.content, title);
    assert.ok(!meta.some((tag) => tag.name === "robots" && tag.content.includes("noindex")));
    assert.match(html, /<html\s[^>]*lang="en"/i);
    const main = html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/i)?.[1];
    assert.ok(main && plain(main).length >= 450, `${path}: meaningful initial HTML, not an empty JavaScript shell`);
    assert.equal([...html.matchAll(/<h1\b/gi)].length, 1, `${path}: exactly one primary heading`);
    assert.ok(!/<h1\b[^>]*style="[^"]*opacity:\s*0/.test(html), `${path}: heading must not be initially hidden`);
    for (const image of tags(html, "img")) assert.ok(Object.hasOwn(image, "alt"), `${path}: all images need an alt attribute (empty is valid for decoration)`);
    const jsonLd = [...html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/gi)].filter((match) => attributes(match[1]).type === "application/ld+json");
    assert.equal(jsonLd.length, 1, `${path}: one controlled JSON-LD graph`);
    const graph = JSON.parse(jsonLd[0][2])["@graph"];
    assert.ok(Array.isArray(graph) && graph.some((item) => item["@type"] === "SoftwareApplication"));
    const software = graph.find((item) => item["@type"] === "SoftwareApplication");
    assert.equal(software.softwareVersion, "2.0.0");
    assert.deepEqual(software.operatingSystem, ["Linux", "macOS", "Windows"]);
    assert.ok(!JSON.stringify(graph).includes("aggregateRating"), "No fabricated rating data");
    if (path !== "/") assert.ok(graph.some((item) => item["@type"] === "BreadcrumbList"));
    if (path === "/faq") {
      const faq = graph.find((item) => item["@type"] === "FAQPage");
      assert.ok(faq?.mainEntity.length >= 6);
      const visible = plain(main);
      for (const item of faq.mainEntity) {
        assert.ok(visible.includes(item.name), "FAQ schema question must exist in page HTML");
        assert.ok(visible.includes(item.acceptedAnswer.text), "FAQ schema answer must exist in page HTML");
      }
    }
    if (path === "/themes") for (const word of ["Catppuccin", "Nord", "Tokyo"]) assert.ok(plain(main).includes(word));
    if (path === "/presets") for (const word of ["Full Info", "Minimal", "Developer", "Gaming"]) assert.ok(plain(main).includes(word));
    for (const anchor of tags(html, "a")) {
      if (!anchor.href || anchor.href.startsWith("mailto:") || anchor.href.startsWith("tel:")) continue;
      const target = resolveRoute(anchor.href, path);
      if (target.url.origin !== origin) continue;
      assert.ok(paths.includes(target.path), `${path}: internal link ${anchor.href} must lead to a prerendered route`);
      if (target.url.hash) {
        const id = decodeURIComponent(target.url.hash.slice(1));
        assert.ok(tags(documents.get(target.path), "[a-z][a-z0-9]*").some((tag) => tag.id === id), `${path}: target anchor ${anchor.href} exists`);
      }
    }
    checks.push(`${path}: substantial JS-less HTML, unique metadata, canonical/social cards, linked routes, accessible images and truthful structured data`);
  }
  const notFound = await get(base ? "/this-page-does-not-exist-termfetch-seo-check" : "/404.html", base ? 404 : 200);
  assert.ok(tags(notFound, "meta").some((tag) => tag.name === "robots" && tag.content.includes("noindex")));
  assert.match(plain(notFound), /not found/i);
  checks.push("Unknown pages have a substantial noindex 404 document (HTTP 404 checked when --base is used)");
  report.status = "Pass";
} catch (error) {
  report.error = String(error);
  process.exitCode = 1;
} finally {
  await writeFile(resolve(evidence, base ? "live-checks.json" : "checks.json"), JSON.stringify(report, null, 2) + "\n");
  console.log(JSON.stringify(report, null, 2));
}
