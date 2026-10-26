# Maintain search and AI-search discoverability

The website publishes readable static HTML for people and crawlers, then hydrates
the same React interface. It does not send a different page to a bot. This work
removes the former empty JavaScript shell; it does not promise a search position,
indexing date or AI citation.

## Build and verify

Run from the repository root:

```sh
npm run build --prefix website
npm run check:seo --prefix website
```

The build checks the Rust catalog, compiles the client, bundles the server render
entry and writes an HTML document for each route:

| Canonical route | Reader intent |
| --- | --- |
| `/` | Understand the product and try the native-style browser studio |
| `/themes` | Compare the 40 source theme palettes |
| `/presets` | Compare Full Info, Minimal, Focused, Developer, Gaming and Custom Modules |
| `/downloads` | Find verified release files, checksums and package-channel status |
| `/docs` | Understand customization concepts and browser/native boundaries |
| `/docs/getting-started` | Follow the initial configuration and export workflow |
| `/releases` | Distinguish published release 2.0.0 from source 2.1.1 |
| `/faq` | Find direct answers to product, installation and privacy questions |

The generated `404.html` is `noindex`. It is not in the sitemap. Nginx must serve
known routes without a redirect to a conflicting trailing-slash URL and return
HTTP 404 for unknown routes. The production configuration is maintained in
[`termfetch-studio-nginx.conf`](../deploy/termfetch-studio-nginx.conf).

To check the actual server after deployment:

```sh
node website/scripts/check-seo.mjs --base https://termfetch-studio.sinansarikaya.dev
```

The static and live checker validate substantial initial HTML, one primary
heading, unique title/description, canonical and social URLs, internal route and
anchor targets, image alt attributes, JSON-LD, shared visible FAQ answers,
robots.txt, sitemap.xml and the optional llms.txt. Live checks additionally
require direct HTTP 200 for canonical pages and HTTP 404 for an unknown page.
They do not assign an invented SEO or GEO score. Results are written under
[`qa/website-revision-seo/`](../qa/website-revision-seo/).

Hydration, interactive fidelity, keyboard operation, mobile overflow and console
errors are separately checked by the real headed browser regression. A static
HTML check is not a browser or accessibility certification.

## Keep facts and page metadata aligned

- [`seo-data.ts`](../website/src/seo-data.ts) owns route metadata, FAQ copy and
  structured data. The visible FAQ imports that same copy.
- [`seo.tsx`](../website/src/seo.tsx) updates metadata after client navigation.
- [`entry-server.tsx`](../website/src/entry-server.tsx) renders the shared Site
  component inside a StaticRouter. It does not execute browser or host APIs.
- [`prerender.mjs`](../website/scripts/prerender.mjs) fills explicit template
  slots, preserving the actual Vite assets and the complete server-rendered body.
- [`distribution.ts`](../website/src/lib/distribution.ts) owns verified download
  URLs and package-manager publication state. Do not enable an install command
  merely because a manifest exists in this repository.

SoftwareApplication data describes the published 2.0.0 application. A separate
SoftwareSourceCode node identifies source 2.1.1. The graph includes truthful
publisher, website, breadcrumb and catalog relationships, with no invented
ratings, reviews, download totals or benchmark claims. The source is MIT licensed
and the application is free; the Offer price is zero. This markup is not a
promise that a software rich result will be displayed.

Keep the eight-route list, sitemap, visible navigation and route content in sync.
Do not add language alternates until complete localized pages exist. The current
site is English and declares `lang="en"`; it does not claim a nonexistent Turkish
locale. Theme and preset galleries need descriptive text as well as visual
swatches. The demo uses sample system data, and real host data must never be used
in promotional screenshots or videos.

## What GEO means here

GEO work is clear, retrievable product evidence: meaningful HTML, useful internal
links, specific questions and answers, accurate release facts, readable catalog
names and structured data that agrees with the page. Google says its normal SEO
foundations also apply to AI features; it does not require special AI files or
schema. Neither indexing nor inclusion is guaranteed.
[Google AI features guidance](https://developers.google.com/search/docs/appearance/ai-features)

The optional llms.txt is a concise navigation document for systems that choose to
use it. Google explicitly says these files do not affect its visibility or
rankings. FAQPage remains descriptive Schema.org vocabulary, but Google removed
its FAQ rich-result feature in May 2026; this project makes no FAQ rich-result
claim. These details were checked against the current documentation on
2026-10-26. [Google documentation updates](https://developers.google.com/search/updates)

OpenAI distinguishes its search crawler from its training crawler. This change
preserves the existing public crawl policy; it does not silently change training
preferences. robots.txt alone cannot prove that a CDN or firewall permits a
crawler's published IP ranges.
[OpenAI crawler documentation](https://developers.openai.com/api/docs/bots)

Microsoft's guidance similarly connects AI visibility to coherent, verifiable
content and canonical signals. Citation visibility is an observation, not a
guaranteed rank.
[Bing on content clarity](https://blogs.bing.com/webmaster/2025/12/Does-Duplicate-Content-Hurt-SEO-and-AI-Search-Visibility/)

## Measure after publication

Use the existing verified Search Console and Bing Webmaster properties, if
available, to submit the sitemap and inspect indexing/crawl reports. Actual
impressions, search queries, clicks and AI citations must be measured there; no
account connection or sitemap submission is claimed by this code change.
Recheck accessibility, image weight and page experience as content changes.
Google's JavaScript guidance recommends server/static rendering because not all
crawlers execute JavaScript.
[Google JavaScript SEO guidance](https://developers.google.com/search/docs/crawling-indexing/javascript/javascript-seo-basics)
