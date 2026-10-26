import { catalog } from "./lib/studio";
import { publicOrigin, release, repository } from "./lib/distribution";

export const FAQ_ITEMS = [
  {
    question: "What is TermFetch Studio?",
    answer: "TermFetch Studio is an open-source Rust terminal UI for customizing Fastfetch themes, presets, modules, colors and ASCII or image logos. It includes a live preview and generates Fastfetch JSONC configuration.",
  },
  {
    question: "Does TermFetch Studio replace Fastfetch?",
    answer: "No. Fastfetch produces the system-information output. TermFetch Studio helps you customize its appearance and layout. Install Fastfetch separately to use native Fastfetch output.",
  },
  {
    question: "How many themes and presets are included?",
    answer: "The current source catalog contains 30 themes and six built-in presets: Full Info, Minimal, Focused, Developer, Gaming and Custom Modules. Full Info contains 25 modules; Custom Modules lets you choose their order and selection.",
  },
  {
    question: "Which operating systems can I download it for?",
    answer: "Published v2.0.0 downloads include a Windows x64 ZIP, a universal macOS archive for Apple Silicon and Intel, Linux x64 glibc and musl archives, and DEB and RPM packages. Terminal image support depends on the terminal and selected image protocol.",
  },
  {
    question: "Can I use my own image as a Fastfetch logo?",
    answer: "Yes. The native studio supports a custom image and image-size choices alongside its ASCII and bundled image catalog. A browser JSONC export needs an image path that exists on your machine and an image protocol supported by your terminal.",
  },
  {
    question: "What does demo mode do?",
    answer: "Demo mode replaces displayed system-information values with sample data for previews and recordings. The browser demo always uses sample values. Before sharing a recording, also check visible file paths and any surrounding desktop content.",
  },
  {
    question: "Can I install TermFetch Studio with a package manager?",
    answer: "Homebrew, WinGet, Scoop and AUR publication is pending. Verified v2.0.0 archives and DEB/RPM files are available on the Downloads page. The current source checkout is version 2.1.0 and may contain changes not included in those published binaries.",
  },
  {
    question: "Is TermFetch Studio free, and do I need an account?",
    answer: "TermFetch Studio is free and open source under the MIT license. The native application runs locally, and no account is needed to try the browser demo. Browser changes stay local until you choose to export a configuration.",
  },
  {
    question: "How do I use a configuration exported from the browser?",
    answer: "Download the generated JSONC, save it as config.jsonc, and run fastfetch --config /path/to/config.jsonc after installing Fastfetch. The browser does not overwrite your system configuration. ASCII artwork is embedded; image paths must point to your local files.",
  },
] as const;

export const ROUTES = [
  { path: "/", name: "Home", title: "TermFetch Studio — Fastfetch themes, image logos and live preview", description: "Make Fastfetch your own with 30 themes, Full Info and custom layouts, image logos and semantic colors. Try the interactive studio for Linux, macOS and Windows." },
  { path: "/themes", name: "Themes", title: "30 Fastfetch themes and color palettes | TermFetch Studio", description: "Explore Catppuccin, Nord, Tokyo Night, Synthwave and more Fastfetch palettes. Compare real colors and try a theme in the interactive TermFetch Studio demo." },
  { path: "/presets", name: "Presets", title: "Fastfetch presets: Full Info, Developer and Gaming | TermFetch Studio", description: "Compare Full Info with 25 modules, Minimal, Focused, Developer, Gaming and Custom Modules. Build a Fastfetch layout with your own colors and logo." },
  { path: "/downloads", name: "Downloads", title: "Download TermFetch Studio for Windows, macOS and Linux", description: "Download verified TermFetch Studio v2.0.0 archives, DEB and RPM packages with SHA-256 checksums. Check the status of Homebrew, WinGet, Scoop and AUR." },
  { path: "/docs", name: "Documentation", title: "Fastfetch customization documentation | TermFetch Studio", description: "Learn how themes, presets, image logos, semantic colors and demo mode work. Understand browser exports and the native TermFetch Studio workflow." },
  { path: "/docs/getting-started", name: "Getting started", title: "Get started with TermFetch Studio and Fastfetch JSONC", description: "Try the studio, export a Fastfetch JSONC file and use it in your terminal. Follow practical setup steps for the browser demo and native Rust application." },
  { path: "/releases", name: "Releases", title: "TermFetch Studio releases and source version status", description: "Find published v2.0.0 downloads and release history. Understand the difference between the current 2.1.0 source and verified public binaries." },
  { path: "/faq", name: "Frequently asked questions", title: "Fastfetch themes, logos and installation FAQ | TermFetch Studio", description: "Answers about TermFetch Studio, Fastfetch requirements, image logos, demo privacy, supported platforms, JSONC exports and package-manager availability." },
] as const;

export function normalizePath(pathname: string): string {
  const path = pathname.split(/[?#]/, 1)[0] || "/";
  return path === "/" ? "/" : path.replace(/\/+$/, "");
}

export function getSeoMetadata(pathname: string) {
  const path = normalizePath(pathname);
  const route = ROUTES.find((item) => item.path === path);
  const canonical = publicOrigin + path;
  const title = route?.title ?? "Page not found | TermFetch Studio";
  const description = route?.description ?? "This TermFetch Studio page could not be found. Explore themes, presets, documentation and verified downloads.";
  const projectId = publicOrigin + "/#project";
  const appId = publicOrigin + "/#software";
  const webpageId = canonical + "#webpage";
  const graph: Record<string, unknown>[] = [
    { "@type": "Organization", "@id": projectId, name: "TermFetch Studio", url: publicOrigin + "/", logo: publicOrigin + "/brand/termfetch-512.png", sameAs: [repository] },
    { "@type": "WebSite", "@id": publicOrigin + "/#website", name: "TermFetch Studio", url: publicOrigin + "/", inLanguage: "en", publisher: { "@id": projectId } },
    { "@type": "SoftwareApplication", "@id": appId, name: "TermFetch Studio", url: publicOrigin + "/", description: "Open-source Rust terminal UI for customizing Fastfetch themes, presets, modules, semantic colors and ASCII or image logos.", applicationCategory: "DeveloperApplication", operatingSystem: ["Linux", "macOS", "Windows"], softwareVersion: release.version, datePublished: release.publishedAt, downloadUrl: publicOrigin + "/downloads", license: repository + "/blob/main/LICENSE", offers: { "@type": "Offer", price: "0", priceCurrency: "USD" }, author: { "@type": "Person", name: "Sinan Sarıkaya", url: "https://github.com/sinansarikaya" } },
    { "@type": "SoftwareSourceCode", "@id": publicOrigin + "/#source", name: "TermFetch Studio source", codeRepository: repository, programmingLanguage: "Rust", version: catalog.version, license: repository + "/blob/main/LICENSE" },
    { "@type": route?.path === "/faq" ? "FAQPage" : "WebPage", "@id": webpageId, name: title, description, url: canonical, inLanguage: "en", isPartOf: { "@id": publicOrigin + "/#website" }, about: { "@id": appId }, ...(path === "/faq" ? { mainEntity: FAQ_ITEMS.map((item) => ({ "@type": "Question", name: item.question, acceptedAnswer: { "@type": "Answer", text: item.answer } })) } : {}) },
  ];
  if (route && path !== "/") {
    const crumbs = [{ name: "Home", item: publicOrigin + "/" }];
    if (path === "/docs/getting-started") crumbs.push({ name: "Documentation", item: publicOrigin + "/docs" });
    crumbs.push({ name: route.name, item: canonical });
    graph.push({ "@type": "BreadcrumbList", "@id": canonical + "#breadcrumb", itemListElement: crumbs.map((crumb, index) => ({ "@type": "ListItem", position: index + 1, name: crumb.name, item: crumb.item })) });
  }
  if (path === "/themes" || path === "/presets") {
    const items = path === "/themes" ? catalog.themes : catalog.presets;
    graph.push({ "@type": "ItemList", "@id": canonical + "#catalog", name: path === "/themes" ? "Fastfetch theme catalog" : "Fastfetch preset catalog", numberOfItems: items.length, itemListElement: items.map((item, index) => ({ "@type": "ListItem", position: index + 1, name: item.name })) });
  }
  return { path, canonical, title, description, image: publicOrigin + "/brand/termfetch-512.png", noindex: !route, structuredData: { "@context": "https://schema.org", "@graph": graph } };
}

export const escapeHtml = (value: string) => value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
export const serializeJsonLd = (value: unknown) => JSON.stringify(value).replace(/</g, "\\u003c").replace(/\u2028/g, "\\u2028").replace(/\u2029/g, "\\u2029");

export function renderSeoHead(pathname: string) {
  const meta = getSeoMetadata(pathname);
  return [
    `<title>${escapeHtml(meta.title)}</title>`,
    `<meta name="description" content="${escapeHtml(meta.description)}">`,
    `<meta name="robots" content="${meta.noindex ? "noindex,follow" : "index,follow,max-image-preview:large"}">`,
    `<link rel="canonical" href="${escapeHtml(meta.canonical)}">`,
    ...Object.entries({ "og:title": meta.title, "og:description": meta.description, "og:type": "website", "og:site_name": "TermFetch Studio", "og:url": meta.canonical, "og:image": meta.image, "og:image:width": "512", "og:image:height": "512", "og:image:alt": "TermFetch Studio TF symbol" }).map(([key, value]) => `<meta property="${key}" content="${escapeHtml(value)}">`),
    ...Object.entries({ "twitter:card": "summary", "twitter:title": meta.title, "twitter:description": meta.description, "twitter:image": meta.image, "twitter:image:alt": "TermFetch Studio TF symbol" }).map(([key, value]) => `<meta name="${key}" content="${escapeHtml(value)}">`),
    `<script id="termfetch-structured-data" type="application/ld+json">${serializeJsonLd(meta.structuredData)}</script>`,
  ].join("\n    ");
}
