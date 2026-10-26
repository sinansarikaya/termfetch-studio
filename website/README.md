# TermFetch Studio website

The application website lives beside the Rust application in this repository.
It contains a local interactive studio, published downloads, getting-started
instructions and release information.

## Development

From the repository root, use a current Node.js 24 release and npm:

```sh
npm ci --prefix website
npm run dev --prefix website
```

The predev and prebuild scripts copy the shared vector and icon assets from
`assets/brand/` into the ignored `website/public/brand/` directory. They do not
create a second brand. The touch icon is the shared 512px PNG, scaled by the
client. See [brand documentation](../docs/brand.md) to regenerate the source icons.

```sh
npm run build --prefix website
npm run lint --prefix website
npm run check:catalog --prefix website
```

Production output is `website/dist/`. After Vite compilation, React renders the
home, downloads, docs, getting-started, releases, themes, presets and FAQ routes
to complete HTML documents. `npm run check:seo --prefix website` checks this
output, per-route metadata, structured data and crawler files. BrowserRouter
hydrates those documents for interactive use. Static hosting serves each route's
`index.html` directly; unknown routes and missing assets return an actual 404.
The repository Dockerfile and [deployment guide](../docs/website-deployment.md)
supply that setup. See [SEO/GEO implementation](../docs/website-seo.md) for
primary-source rationale and verification.

## App catalog and Fastfetch export

`src/lib/catalog.json` is a reviewed snapshot extracted from the Rust theme,
preset, module, logo and settings sources. `catalog-sources.json` records the
source hashes; `check-catalog.mjs` fails when those files drift. It also checks
480 theme/preset/demo combinations and color, ordering and logo export rules.
Run the build from a full repository checkout, because these source files are
required inputs. Review source changes before refreshing hashes or the catalog.

The browser uses the app’s catalog and configuration rules, with native-style
terminal chrome, module glyphs and semantic color groups. Its initial marketing
showcase combines Full Info, Synthwave and Pikachu; the extracted Rust defaults
remain unchanged. This is a browser implementation, not execution of the native
binary. Terminal geometry and image protocols are represented in browser layout. All preview specs are sample fixtures, including when demo
export is disabled; Fastfetch detects real specs only when the exported file is
run on the visitor’s machine.

Apply locally saves to local storage. Restore browser save validates every
persisted field. No host config, terminal colors, shell startup or backups are
changed. Color edits have an explicit Apply/Cancel/Reset flow. Export is disabled
while a color draft is open. ASCII exports embed the art as Fastfetch `data`;
image exports require a supplied local path, otherwise automatic distro art is
used. The native writer instead manages image/ASCII files and exact terminal
geometry. These adaptations are intentional and disclosed in the site’s Docs.

## Releases and package channels

`src/lib/distribution.ts` is the single source of truth. Published v2.0.0 assets
and the checksum file were verified against the repository’s GitHub release.
Cargo source v2.1.1 is displayed separately; it is not advertised as a published
binary release. Homebrew, WinGet, Scoop and AUR stay `pending` until their actual
registry publication is verified. The typed `available` state requires both a
real registry URL and an installation command. Downloadable DEB/RPM packages do
not imply that APT/DNF/Zypper repositories exist.

Update the registry when publishing a release. Verify asset URLs, checksums and
release dates before changing the public copy. No release/tag or package registry
is created by website scripts.

## Browser regression

Build first, then run:

```sh
npm run check --prefix website
```

The regression starts a local static server with SPA fallback and launches one
**headed** Chromium browser. It records its real desktop window on Linux,
interactions, screenshots and a Playwright trace under the ignored `website/qa/`
directory. A display/launch failure is Blocked; an assertion failure is Fail.
There is no headless fallback.

The current workstation’s installed Playwright and Chromium paths are defaults.
On another machine supply `PLAYWRIGHT_MODULE` (absolute path to an ESM Playwright
entry point) and `PLAYWRIGHT_CHROMIUM_EXECUTABLE`. Use `BASE_URL` to run the same
checks against a local production container or the published site. `xwininfo`
is required for the Linux headed-window evidence check.

Assertions cover defaults, theme/preset/ASCII/image/size changes, module toggles
and order, semantic color draft/apply/cancel/reset, local apply/restore and invalid
storage, a real JSONC download with actual ESC codes, scoped keyboard shortcuts,
verified download links, Docs anchors, skip links, direct routes/reloads, local
assets, console/page errors and page overflow at 320/390/768/1440px. The regression
does not establish native Windows/macOS compatibility or legal accessibility
compliance.

## Public metadata

The configured production origin is `https://termfetch-studio.sinansarikaya.dev`.
`index.html`, the runtime canonical URL, `public/robots.txt` and `public/sitemap.xml`
use this origin. Social metadata uses the shared raster logo. Route content is
rendered by React; this implementation does not claim server-side prerendering.
