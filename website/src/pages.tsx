import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { catalog } from "./lib/studio";
import {
  packageChannels,
  release,
  releaseAssets,
  repository,
  sourceCommands,
} from "./lib/distribution";

function PageIntro({
  label,
  title,
  children,
}: {
  label: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <div className="page-intro">
      <p className="eyebrow">{label}</p>
      <h1>{title}</h1>
      <p>{children}</p>
    </div>
  );
}

export function Downloads() {
  return (
    <div className="document">
      <PageIntro label="DISTRIBUTION / DOWNLOADS" title="Get TermFetch Studio.">
        One studio. Linux, macOS and Windows. Download a published build or
        explore the current source.
      </PageIntro>
      <div className="release-summary">
        <div>
          <span className="eyebrow">PUBLISHED RELEASE</span>
          <h2>v{release.version}</h2>
          <p>
            Published{" "}
            <time dateTime={release.publishedAt}>{release.publishedAt}</time>
          </p>
        </div>
        <a href={release.url}>Release notes ↗</a>
      </div>
      <div className="download-grid">
        {releaseAssets.map((asset) => (
          <article className="download-card" key={asset.file}>
            <p className="eyebrow">
              {asset.platform} / {asset.format}
            </p>
            <h2>{asset.title}</h2>
            <p>{asset.note}</p>
            <a className="download-link" href={asset.url}>
              Download v{release.version} <span aria-hidden="true">↓</span>
            </a>
          </article>
        ))}
      </div>
      <div className="note checksum-note">
        <strong>Verify your download</strong>
        <p>
          Compare the SHA-256 of your archive or package with the matching entry
          in the release checksum file.
        </p>
        <a href={release.checksums}>SHA256SUMS.txt ↗</a>
        <p>
          <code>sha256sum your-downloaded-file</code> (Linux),{" "}
          <code>shasum -a 256 your-downloaded-file</code> (macOS), or{" "}
          <code>Get-FileHash your-downloaded-file -Algorithm SHA256</code>{" "}
          (PowerShell).
        </p>
      </div>
      <h2>Package managers</h2>
      <p>
        Registry distribution is coming next. Downloadable DEB and RPM files
        above are available; a managed APT, DNF or Zypper repository is not
        announced here.
      </p>
      <div className="channel-table">
        <table>
          <caption>Package channel registry</caption>
          <thead>
            <tr>
              <th>Channel</th>
              <th>Platform</th>
              <th>Format</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {packageChannels.map((channel) => (
              <tr key={channel.name}>
                <th scope="row">{channel.name}</th>
                <td>{channel.platform}</td>
                <td>{channel.format}</td>
                <td>
                  {channel.status === "pending" ? (
                    <span className="pending">Coming soon</span>
                  ) : (
                    <a href={channel.url}>
                      <code>{channel.command}</code>
                    </a>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <h2>Build current source · v{catalog.version}</h2>
      <p>
        The source checkout can be ahead of the published release. Use a current
        Rust toolchain with edition 2024 support, Git and a platform linker.
        Keep the checkout’s assets directory available. Fastfetch is a separate
        prerequisite for native output; a Nerd Font is recommended for module
        glyphs.
      </p>
      <pre tabIndex={0}>{sourceCommands}</pre>
      <p>
        First start can set up shell and terminal integration. Read the{" "}
        <a href={`${repository}#installation`}>native installation guide</a>{" "}
        before running the application.
      </p>
      <Link to="/docs/getting-started">Getting started →</Link>
    </div>
  );
}

export function Docs({ gettingStarted = false }: { gettingStarted?: boolean }) {
  if (!gettingStarted)
    return (
      <div className="document">
        <PageIntro label="DOCS / REFERENCE" title="Know your studio.">
          How themes, presets, modules, semantic colors and logos work together
          in TermFetch Studio.
        </PageIntro>
        <div className="docs-overview">
          <section>
            <h2>Themes set the visual foundation.</h2>
            <p>
              The 30 source palettes define a background, readable text, accents
              and five semantic groups: OS, desktop, hardware, network and time.
              Use a complete theme first, then override individual groups when
              you want a particular contrast.
            </p>
            <Link to="/themes">Explore the theme library →</Link>
          </section>
          <section>
            <h2>Presets compose the information.</h2>
            <p>
              Full Info includes all 25 modules. Minimal, Focused, Developer and
              Gaming offer more focused layouts. Custom Modules is built from
              the items you enable and the order you choose. Preset copies
              retain section breaks and color-dot settings.
            </p>
            <Link to="/presets">Compare the layouts →</Link>
          </section>
          <section>
            <h2>Colors follow a clear hierarchy.</h2>
            <p>
              A semantic group override colors both its keys and output values.
              Inherit follows global icon and text settings, which can use the
              theme palette. Apply, Cancel and Reset let you preview a color
              change without committing it.
            </p>
          </section>
          <section>
            <h2>Artwork depends on the terminal.</h2>
            <p>
              ASCII artwork is text and travels inside the browser export. Image
              artwork uses a local file path, size and protocol such as Kitty,
              Sixel or a text-rendered fallback. The native app sizes images
              using the real terminal cell geometry; browser rendering is an
              interactive recreation.
            </p>
          </section>
          <section id="behavior" tabIndex={-1}>
            <h2>Local by design. Sample data in the browser.</h2>
            <p>
              Browser Apply saves locally and never writes terminal
              configuration. Every browser preview uses samples; turning Demo
              export off lets Fastfetch detect actual values when you run the
              exported JSONC. Native Apply, shell setup, backups and Doctor run
              only in the installed application.
            </p>
          </section>
        </div>
        <Link className="button-primary" to="/docs/getting-started">
          Follow the step-by-step guide →
        </Link>
      </div>
    );
  return (
    <div className="document">
      <PageIntro
        label="DOCS / GETTING STARTED"
        title={gettingStarted ? "From studio to shell." : "Know your studio."}
      >
        Explore in the browser. Take your Fastfetch config with you. Run the
        native studio when you are ready.
      </PageIntro>
      <nav className="doc-toc" aria-label="Documentation sections">
        <a href="#export">Use a browser export</a>
        <a href="#native">Run the native studio</a>
        <a href="#behavior">Know the boundaries</a>
      </nav>
      <section id="export" tabIndex={-1}>
        <h2>01 / Use a browser export</h2>
        <ol>
          <li>
            Choose a theme, preset, logo and image size. Modules lets you toggle
            and reorder the Custom Modules layout; use the current preset as a
            starting point when useful.
          </li>
          <li>
            Open Settings → Edit colors. Group colors override both module keys
            and values. Apply colors keeps the edit; Cancel discards it; Reset
            restores inherited theme colors.
          </li>
          <li>
            Decide whether Demo mode should remain on. The browser always
            previews samples; with Demo mode off, the exported config omits
            fixed module formats so Fastfetch reports your system.
          </li>
          <li>
            For PNG logos, enter an existing image path and choose a protocol
            supported by your terminal. Without a path the export uses
            Fastfetch’s automatic distro logo. ASCII is embedded directly.
          </li>
          <li>
            Download JSONC, save it as <code>config.jsonc</code>, and run the
            command below after installing Fastfetch separately.
          </li>
        </ol>
        <pre tabIndex={0}>fastfetch --config /path/to/config.jsonc</pre>
        <p>
          The download does not overwrite an existing configuration. Back up
          your file before replacing Fastfetch’s default config. Nerd Font
          glyphs need a compatible terminal font.
        </p>
      </section>
      <section id="native" tabIndex={-1}>
        <h2>02 / Run the native studio</h2>
        <p>
          <Link to="/downloads">Download published v{release.version}</Link> or
          build source v{catalog.version}. The current source defaults to
          Catppuccin Mocha, Full Info, Arch ASCII, Medium image sizing and demo
          mode on. An older release may differ.
        </p>
        <pre tabIndex={0}>
          {
            "termfetch-studio\ntermfetch-studio --no-demo\ntermfetch-studio doctor"
          }
        </pre>
        <p>
          From a source checkout, run{" "}
          <code>./target/release/termfetch-studio</code> on Linux/macOS or{" "}
          <code>.\target\release\termfetch-studio.exe</code> in Windows
          PowerShell.
        </p>
        <p>
          Native navigation supports Tab / Shift+Tab, arrow keys, Enter, mouse
          and wheel. Use <kbd>?</kbd> for shortcuts, <kbd>p</kbd> for real
          Fastfetch preview, <kbd>a</kbd> to apply and <kbd>d</kbd> for demo
          mode. Modules supports Space to toggle and <kbd>J</kbd>/<kbd>K</kbd>{" "}
          to reorder.
        </p>
        <p>
          Native Apply saves settings and writes the studio and Fastfetch
          configs. Default paths are{" "}
          <code>~/.config/termfetch-studio/fastfetch.jsonc</code> and{" "}
          <code>~/.config/fastfetch/config.jsonc</code>; platform config
          locations and XDG settings can change these. The app preserves a
          foreign config before first overwrite. Native Settings includes backup
          and diagnostic tools. See the{" "}
          <a href={`${repository}/blob/main/README.md`}>
            repository uninstall guide
          </a>{" "}
          to remove shell and terminal integration.
        </p>
      </section>
      <section id="behavior" tabIndex={-1}>
        <h2>03 / Browser boundaries</h2>
        <p>
          Apply locally stores the selection in local storage. Restore browser
          save explicitly reloads it; new sessions start with the Rust defaults.
          Clearing site data removes that save.
        </p>
        <p>
          Preview values are privacy-safe Rust demo fixtures. No hardware
          probes, filesystem writes, shell integration, backups or terminal
          color changes happen in the browser. Images are visually approximated;
          native sizing uses actual image and terminal cell geometry.
        </p>
        <p>
          Exports embed ASCII with Fastfetch’s data logo type instead of the
          Rust writer’s managed file path. Image export uses your supplied path
          and a maximum width; Fastfetch determines height. Module order,
          section breaks, key width, separator, title and semantic colors follow
          the Rust writer.
        </p>
        <p>
          Focus the studio and press <kbd>1</kbd>–<kbd>6</kbd> to switch
          sections, <kbd>a</kbd> to apply locally, <kbd>d</kbd> to toggle demo
          export, <kbd>o</kbd> to switch output and <kbd>?</kbd> for help.
          Shortcuts never intercept typing inside inputs, selects or textareas.
        </p>
      </section>
    </div>
  );
}

export function Releases() {
  return (
    <div className="document">
      <PageIntro label="RELEASES / HISTORY" title="Built to keep evolving.">
        Published downloads and current source are shown separately, so you know
        what you are getting.
      </PageIntro>
      <article className="release-entry">
        <p className="eyebrow">PUBLISHED ON {release.publishedAt}</p>
        <h2>
          v{release.version} <span className="available">Available</span>
        </h2>
        <p>
          Linux archives and DEB/RPM packages, a macOS universal archive and a
          Windows x64 ZIP are available on GitHub. Check the original release
          notes for version-specific changes and installation guidance.
        </p>
        <div className="inline-links">
          <Link to="/downloads">Download v{release.version} →</Link>
          <a href={release.url}>Official release notes ↗</a>
          <a href={release.checksums}>SHA-256 checksums ↗</a>
        </div>
      </article>
      <article className="release-entry">
        <p className="eyebrow">CURRENT SOURCE CHECKOUT</p>
        <h2>
          v{catalog.version}{" "}
          <span className="pending">Source ahead of release</span>
        </h2>
        <p>
          The source includes the Rust studio, theme and logo catalogs, module
          builder, semantic colors, demo mode and Fastfetch JSONC generation.
          This is source state, not a claim that v{catalog.version} binaries
          have been released.
        </p>
        <a href={repository}>Browse source ↗</a>
      </article>
      <h2>What comes next</h2>
      <p>
        {packageChannels
          .filter((channel) => channel.status === "pending")
          .map((channel) => channel.name)
          .join(", ")}{" "}
        publication is pending. The{" "}
        <Link to="/downloads">Downloads registry</Link> will provide
        installation commands when those channels are verified live.
      </p>
      <h2>Platform scope</h2>
      <p>
        The project targets Linux, macOS and Windows. Behavior depends on the
        terminal, Fastfetch and platform integration. Source support is not a
        verified compatibility claim for every distribution or emulator.
      </p>
      <a href={`${repository}/releases`}>Full release history ↗</a>
    </div>
  );
}
