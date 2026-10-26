import { useEffect, useState } from "react";
import type { CSSProperties } from "react";
import {
  BrowserRouter,
  Link,
  NavLink,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import { catalog, getTheme } from "./lib/studio";
import { packageChannels, release } from "./lib/distribution";
import { Downloads, Docs, Releases } from "./pages";
import {
  NativePreview,
  NativeStudio,
  ThemeArt,
  ThemeSwatches,
} from "./NativeStudio";
import { LOOKS, settingsForLook } from "./lib/looks";
import type { Look } from "./lib/looks";
import { Seo } from "./seo";
import { FAQ_ITEMS } from "./seo-data";
import "./App.css";

function Brand() {
  return (
    <>
      <img src="/brand/termfetch-symbol.svg" alt="" width="32" height="32" />
      <span>
        TermFetch <b>Studio</b>
      </span>
    </>
  );
}
export function Site() {
  const location = useLocation();
  useEffect(() => {
    if (location.hash)
      document.getElementById(location.hash.slice(1))?.scrollIntoView();
    else window.scrollTo(0, 0);
  }, [location.pathname, location.hash]);
  return (
    <>
      <Seo />
      <a
        className="skip"
        href="#main"
        onClick={(e) => {
          e.preventDefault();
          document.getElementById("main")?.focus();
        }}
      >
        Skip to content
      </a>
      <header className="site-header">
        <Link to="/" className="brand" aria-label="TermFetch Studio home">
          <Brand />
        </Link>
        <nav aria-label="Main navigation">
          <NavLink to="/themes">Themes</NavLink>
          <NavLink to="/presets">Presets</NavLink>
          <NavLink to="/docs">Docs</NavLink>
          <NavLink to="/downloads">Downloads</NavLink>
        </nav>
        <a
          className="source-link"
          href="https://github.com/sinansarikaya/termfetch-studio"
        >
          GitHub ↗
        </a>
      </header>
      <main id="main" tabIndex={-1}>
        <Routes>
          <Route path="/" element={<Home key={location.search} />} />
          <Route path="/themes" element={<ThemePage />} />
          <Route path="/presets" element={<PresetPage />} />
          <Route path="/downloads" element={<Downloads />} />
          <Route path="/docs" element={<Docs />} />
          <Route
            path="/docs/getting-started"
            element={<Docs gettingStarted />}
          />
          <Route path="/releases" element={<Releases />} />
          <Route path="/faq" element={<FAQPage />} />
          <Route
            path="*"
            element={
              <div className="document">
                <h1>Page not found</h1>
                <Link to="/">Return to the studio</Link>
              </div>
            }
          />
        </Routes>
      </main>
      <footer className="site-footer">
        <div>
          <Link to="/" className="brand">
            <Brand />
          </Link>
          <p>
            A little more personality.
            <br />
            Every time you open your terminal.
          </p>
          <span className="footer-meta">Open source · MIT · Built in Rust</span>
        </div>
        <div>
          <h3>Explore</h3>
          <Link to="/themes">40 themes</Link>
          <Link to="/presets">Presets & layouts</Link>
          <Link to="/downloads">Downloads</Link>
        </div>
        <div>
          <h3>Learn</h3>
          <Link to="/docs/getting-started">Getting started</Link>
          <Link to="/docs">Documentation</Link>
          <Link to="/faq">FAQ</Link>
        </div>
        <div>
          <h3>Build with us</h3>
          <a href="https://github.com/sinansarikaya/termfetch-studio">
            GitHub ↗
          </a>
          <Link to="/releases">Releases</Link>
          <small>
            Released v{release.version}
            <br />
            Source v{catalog.version}
          </small>
        </div>
      </footer>
    </>
  );
}
function SectionHeading({
  eyebrow,
  title,
  text,
}: {
  eyebrow: string;
  title: string;
  text?: string;
}) {
  return (
    <div className="section-intro">
      <p className="eyebrow">{eyebrow}</p>
      <h2>{title}</h2>
      {text && <p>{text}</p>}
    </div>
  );
}
function ThemeGallery({
  all = false,
  onSelect,
}: {
  all?: boolean;
  onSelect?: (id: string) => void;
}) {
  const [filter, setFilter] = useState("all");
  const showcaseIds = [
    "synthwave",
    "catppuccin-mocha",
    "tokyo-night",
    "dracula",
    "nord",
    "matrix",
    "rose-pine",
    "gruvbox-dark",
    "catppuccin-latte",
    "cotton-candy",
    "cyberpunk",
    "sakura",
  ];
  const themes = (
    all
      ? catalog.themes
      : catalog.themes.filter((t) => showcaseIds.includes(t.id))
  ).filter((t) => filter === "all" || t.category === filter);
  return (
    <>
      <div className="gallery-toolbar">
        <span>
          {all
            ? `${catalog.themes.length} palettes. All yours.`
            : "Real palettes from the app catalog."}
        </span>
        {all ? (
          <select
            aria-label="Theme gallery category"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
          >
            <option value="all">All themes</option>
            {["Community", "Dark", "Light", "Pastel", "PopCulture"].map((c) => (
              <option key={c}>{c}</option>
            ))}
          </select>
        ) : (
          <Link to="/themes">Explore all 40 themes →</Link>
        )}
      </div>
      <div className="marketing-theme-grid">
        {themes.map((t) => (
          <article className="marketing-theme-card" key={t.id}>
            <ThemeArt theme={t} />
            <div>
              <h3>{t.name}</h3>
              <ThemeSwatches theme={t} />
            </div>
            {onSelect ? (
              <button
                onClick={() => onSelect(t.id)}
                aria-label={`Try ${t.name}`}
              >
                Try this palette ↗
              </button>
            ) : (
              <Link to={"/?theme=" + t.id + "#studio"}>
                Try this palette ↗
              </Link>
            )}
          </article>
        ))}
      </div>
    </>
  );
}
function LookCards({ onSelect }: { onSelect?: (look: Look) => void }) {
  return (
    <div className="look-grid">
      {LOOKS.map((look) => {
        const settings = settingsForLook(look),
          theme = getTheme(settings);
        return (
          <article
            className="look-card"
            key={look.id}
            style={{ "--look-accent": theme.primary } as CSSProperties}
          >
            <div className="look-preview">
              <NativePreview settings={settings} mini />
            </div>
            <div className="look-copy">
              <span className="look-caption">
                {look.preset === "full"
                  ? "25 MODULES · FULL INFO"
                  : look.preset.toUpperCase() + " LAYOUT"}
              </span>
              <h3>{look.name}</h3>
              <p>{look.description}</p>
              {onSelect ? (
                <button onClick={() => onSelect(look)}>
                  Load this look into the studio →
                </button>
              ) : (
                <Link to={"/?look=" + look.id + "#studio"}>
                  Try this complete look →
                </Link>
              )}
            </div>
          </article>
        );
      })}
    </div>
  );
}
function FAQ({ full = false }: { full?: boolean }) {
  return (
    <div className="faq-list">
      {(full ? FAQ_ITEMS : FAQ_ITEMS.slice(0, 5)).map((item) => (
        <details key={item.question}>
          <summary>
            {item.question}
            <span aria-hidden="true">+</span>
          </summary>
          <p>{item.answer}</p>
        </details>
      ))}
    </div>
  );
}
function Home() {
  const location = useLocation();
  const [look, setLook] = useState<Look>(LOOKS[0]);
  useEffect(() => {
    const query = new URLSearchParams(location.search),
      found = LOOKS.find((item) => item.id === query.get("look")) || LOOKS[0];
    const theme = query.get("theme"),
      requestedPreset = query.get("preset");
    let next =
      theme && catalog.themes.some((t) => t.id === theme)
        ? { ...found, id: "theme-" + theme, theme }
        : found;
    if (
      requestedPreset &&
      catalog.presets.some((item) => item.id === requestedPreset)
    ) {
      next = {
        ...next,
        id: next.id + "-preset-" + requestedPreset,
        preset: requestedPreset,
      };
    }
    let active = true;
    queueMicrotask(() => {
      if (active) setLook(next);
    });
    return () => {
      active = false;
    };
  }, [location.search]);
  function chooseLook(next: Look) {
    setLook(next);
    document
      .getElementById("studio")
      ?.scrollIntoView({ behavior: "smooth", block: "start" });
  }
  const initial = settingsForLook(look);
  return (
    <>
      <section className="hero marketing-container">
        <div className="hero-orb" aria-hidden="true" />
        <p className="hero-kicker">
          <span className="live-dot" /> YOUR TERMINAL IS PERSONAL. MAKE IT LOOK
          THAT WAY.
        </p>
        <h1>
          System info.
          <br />
          <span>With a signature.</span>
        </h1>
        <p className="hero-description">
          Turn Fastfetch into a setup that feels like you. Choose a palette,
          bring your favorite artwork, and put every detail exactly where you
          want it.
        </p>
        <div className="hero-actions">
          <a className="button-primary" href="#studio">
            Try the live studio <span>↓</span>
          </a>
          <Link className="button-secondary" to="/downloads">
            Download for your platform ↗
          </Link>
        </div>
        <div className="hero-proof">
          <span>
            <b>40</b> real palettes
          </span>
          <span>
            <b>25</b> configurable modules
          </span>
          <span>ASCII + image logos</span>
          <span>Linux · macOS · Windows</span>
        </div>
        <div className="hero-palette" aria-hidden="true">
          {catalog.themes.slice(0, 15).map((t) => (
            <i key={t.id} style={{ background: t.primary }} />
          ))}
        </div>
      </section>
      <section className="demo-section marketing-wide">
        <div className="demo-introduction">
          <div>
            <p className="eyebrow">THIS IS YOUR PLAYGROUND</p>
            <h2>Don’t imagine it. Make it.</h2>
          </div>
          <p>
            Click around. Change everything. The app’s real themes, presets and
            module rules power this interactive browser studio.
          </p>
        </div>
        <div className="look-switcher" aria-label="Curated demo looks">
          {LOOKS.map((item) => (
            <button
              key={item.id}
              onClick={() => setLook(item)}
              aria-pressed={look.id === item.id}
            >
              {item.preset === "full"
                ? "Full Info / Neon"
                : item.preset[0].toUpperCase() + item.preset.slice(1)}
              <span>{item.logo === "pikachu" ? "Pikachu" : item.logo}</span>
            </button>
          ))}
        </div>
        <NativeStudio key={look.id} initial={initial} />
        <p className="showcase-disclosure">
          Curated showcase:{" "}
          {catalog.themes.find((t) => t.id === look.theme)?.name} ·{" "}
          {catalog.presets.find((p) => p.id === look.preset)?.name} · all five
          semantic groups colored. Native app defaults remain available in
          Settings.
        </p>
      </section>
      <div className="capability-strip">
        <span>Rust + Ratatui</span>
        <i>✦</i>
        <span>Keyboard-first</span>
        <i>✦</i>
        <span>Human-readable JSONC</span>
        <i>✦</i>
        <span>No account. No cloud setup.</span>
        <i>✦</i>
        <span>Make it yours.</span>
      </div>
      <section className="marketing-container marketing-section">
        <SectionHeading
          eyebrow="01 / A PALETTE FOR EVERY KIND OF YOU"
          title="Same terminal. Entirely different energy."
          text="From quiet Catppuccin mornings to neon Synthwave nights. These aren’t decorative website swatches — they’re the actual palettes your Fastfetch setup will use."
        />
        <ThemeGallery
          onSelect={(theme) =>
            chooseLook({ ...look, id: "theme-" + theme, theme })
          }
        />
      </section>
      <section className="marketing-container marketing-section">
        <SectionHeading
          eyebrow="02 / YOUR INFORMATION, YOUR COMPOSITION"
          title="A little less noise. Or the whole picture."
          text="Full Info brings all 25 modules together. Developer, Gaming and Minimal give you a different starting point. Try a complete look below — every choice loads into the live studio."
        />
        <LookCards onSelect={chooseLook} />
        <Link className="section-link" to="/presets">
          Compare all six presets and their modules →
        </Link>
      </section>
      <section className="marketing-container marketing-section identity-section">
        <div>
          <p className="eyebrow">03 / NOT JUST A DISTRO LOGO</p>
          <h2>
            Put a little
            <br />
            <span>you</span> in the prompt.
          </h2>
          <p>
            Keep it classic with crisp ASCII. Go bold with image artwork. Find
            the right scale, then make the palette sing around it.
          </p>
          <ul className="check-list">
            <li>Built-in ASCII and bundled image choices</li>
            <li>Small, Medium and Large logo sizing</li>
            <li>Custom image paths and terminal protocols</li>
            <li>Per-group colors that tie the whole setup together</li>
          </ul>
          <button className="text-action" onClick={() => chooseLook(LOOKS[0])}>
            Try Full Info with Pikachu →
          </button>
        </div>
        <div className="identity-visual">
          <div className="image-logo-stage">
            <span>IMAGE / MEDIUM</span>
            <img
              src="/preset-logos/pikachu.png"
              alt="Pikachu image logo from the bundled catalog"
              width="240"
              height="260"
            />
            <ThemeSwatches
              theme={catalog.themes.find((t) => t.id === "synthwave")!}
            />
          </div>
          <div className="ascii-logo-stage">
            <span>ASCII / NATIVE CRISP</span>
            <pre>{catalog.logos.find((l) => l.id === "arch")?.art}</pre>
            <small>Small footprint. Big personality.</small>
          </div>
          <div className="size-visual">
            <span>One image. Your scale.</span>
            {[42, 65, 92].map((size, i) => (
              <div key={size}>
                <img
                  src="/preset-logos/pikachu.png"
                  width={size}
                  height={size}
                  alt=""
                />
                <small>{["S", "M", "L"][i]}</small>
              </div>
            ))}
          </div>
        </div>
      </section>
      <section className="marketing-container marketing-section">
        <SectionHeading
          eyebrow="04 / LOOKS LIKE YOUR MACHINE. FEELS LIKE YOURS."
          title="From a default readout to a considered setup."
        />
        <div className="before-after">
          <article>
            <span className="comparison-label">BEFORE / A STARTING POINT</span>
            <pre>
              {
                "user@machine\n------------\nOS: Linux\nKernel: 6.x\nShell: zsh\nCPU: Your CPU\nMemory: Your memory\n\nA useful readout. Room for you."
              }
            </pre>
          </article>
          <article className="comparison-after">
            <span className="comparison-label">
              AFTER / YOUR PALETTE, ARTWORK & LAYOUT
            </span>
            <NativePreview settings={settingsForLook(LOOKS[0])} mini />
          </article>
        </div>
        <p className="comparison-caption">
          Illustrative sample output. Your chosen modules report your actual
          system when you run Fastfetch locally.
        </p>
      </section>
      <section className="marketing-container marketing-section">
        <div className="feature-grid">
          <article>
            <span className="feature-icon">󰅩</span>
            <h3>Every module earns its place.</h3>
            <p>
              Keep the essentials. Add the details you care about. Toggle and
              reorder 25 modules without hand-editing a configuration file.
            </p>
            <div className="module-feature-list">
              {["Operating System", "CPU", "GPU", "Memory"].map((name, i) => (
                <span key={name}>
                  <b>{["✓", "✓", "✓", "·"][i]}</b>
                  {name}
                  <small>↕</small>
                </span>
              ))}
            </div>
            <Link to="/presets">Explore the layouts →</Link>
          </article>
          <article>
            <span className="feature-icon">󰏘</span>
            <h3>Color with intention.</h3>
            <p>
              OS, desktop, hardware, network and time each have a semantic
              color. Fine-tune icons and text, or let the palette do the work.
            </p>
            <div className="semantic-feature">
              {groupsForMarketing.map((name, i) => (
                <span key={name}>
                  <i
                    style={{
                      background: [
                        "#ffcf70",
                        "#81f4ff",
                        "#ffd700",
                        "#ff8ab4",
                        "#b7e99e",
                      ][i],
                    }}
                  />
                  {name}
                  <code>
                    {["#FFCF70", "#81F4FF", "#FFD700", "#FF8AB4", "#B7E99E"][i]}
                  </code>
                </span>
              ))}
            </div>
          </article>
          <article>
            <span className="feature-icon">◉</span>
            <h3>Show your style. Keep your details.</h3>
            <p>
              Demo mode swaps system values for sample specs. Explore,
              screenshot and record a look without turning your hardware or
              network details into the centerpiece.
            </p>
            <div className="privacy-feature">
              <span>Network</span>
              <s>your real address</s>
              <strong>203.0.113.42</strong>
              <small>SAMPLE DATA · DEMO MODE</small>
            </div>
            <Link to="/docs#behavior">Understand demo privacy →</Link>
          </article>
        </div>
      </section>
      <section className="marketing-container marketing-section steps-section">
        <SectionHeading
          eyebrow="05 / FROM FIRST CLICK TO YOUR NEXT PROMPT"
          title="Make something you want to open again."
        />
        <div className="steps-grid">
          <article>
            <b>01</b>
            <h3>Find your starting point.</h3>
            <p>
              Pick a theme and preset. Browse logos, switch the size, and see
              the composition change live.
            </p>
          </article>
          <article>
            <b>02</b>
            <h3>Make the details yours.</h3>
            <p>
              Choose modules and semantic colors. Keep Full Info, or build a
              quieter custom layout.
            </p>
          </article>
          <article>
            <b>03</b>
            <h3>Take it to your terminal.</h3>
            <p>
              Download TermFetch Studio, or export JSONC from the browser and
              run it with Fastfetch.
            </p>
            <Link to="/docs/getting-started">
              Follow the getting-started guide →
            </Link>
          </article>
        </div>
      </section>
      <section className="marketing-container marketing-section platform-section">
        <div>
          <p className="eyebrow">ONE STUDIO. THREE PLATFORMS.</p>
          <h2>
            Your next setup
            <br />
            starts here.
          </h2>
          <p>
            Published v{release.version} downloads are ready. Current source v
            {catalog.version} is maintained alongside this website.
          </p>
          <Link className="button-primary" to="/downloads">
            Choose your download ↗
          </Link>
        </div>
        <div className="platform-cards">
          {[
            {
              name: "Windows",
              detail: "x64 ZIP · extract/run EXE · Fastfetch separate",
              symbol: "⊞",
            },
            {
              name: "macOS",
              detail: "Universal · Apple Silicon + Intel",
              symbol: "⌘",
            },
            { name: "Linux", detail: "glibc · musl · DEB · RPM", symbol: "󰌽" },
          ].map((p) => (
            <Link key={p.name} to="/downloads">
              <span>{p.symbol}</span>
              <strong>{p.name}</strong>
              <small>{p.detail}</small>
              <i>↗</i>
            </Link>
          ))}
          <p>
            {packageChannels.map((c) => c.name).join(" · ")}{" "}
            <span>coming soon</span>
          </p>
        </div>
      </section>
      <section className="marketing-container marketing-section faq-section">
        <SectionHeading
          eyebrow="A FEW GOOD QUESTIONS"
          title="Before you make it yours."
        />
        <FAQ />
        <Link className="section-link" to="/faq">
          All questions & answers →
        </Link>
      </section>
      <section className="final-cta marketing-container">
        <div className="cta-colors" aria-hidden="true">
          {catalog.themes.slice(0, 7).map((t) => (
            <i key={t.id} style={{ background: t.primary }} />
          ))}
        </div>
        <p className="eyebrow">A SMALL CHANGE TO AN EVERYDAY THING.</p>
        <h2>
          Open your terminal.
          <br />
          <span>Feel at home.</span>
        </h2>
        <div className="hero-actions">
          <a className="button-primary" href="#studio">
            Make your setup ↑
          </a>
          <Link className="button-secondary" to="/downloads">
            Download TermFetch Studio ↗
          </Link>
        </div>
      </section>
    </>
  );
}
const groupsForMarketing = [
  "Operating system",
  "Desktop",
  "Hardware",
  "Network",
  "Time",
];
function ThemePage() {
  return (
    <div className="marketing-container route-page">
      <div className="route-intro">
        <p className="eyebrow">THE PALETTE LIBRARY</p>
        <h1>
          Thirty ways
          <br />
          to set the mood.
        </h1>
        <p>
          Dark, light, pastel and playful. Every palette below comes directly
          from the Rust app’s catalog, including its semantic group colors.
          Choose one to try in the live studio.
        </p>
      </div>
      <ThemeGallery all />
      <div className="route-bottom">
        <Link className="button-primary" to="/#studio">
          Open the studio →
        </Link>
      </div>
    </div>
  );
}
function PresetPage() {
  return (
    <div className="marketing-container route-page">
      <div className="route-intro">
        <p className="eyebrow">PRESETS & COMPOSITION</p>
        <h1>
          The whole picture.
          <br />
          Or only what matters.
        </h1>
        <p>
          Presets choose which modules appear, in what order, and how they’re
          grouped. Start with a ready-made layout, then use Custom Modules to
          make it your own.
        </p>
      </div>
      <LookCards />
      <div className="preset-comparison">
        {catalog.presets.map((p) => (
          <article key={p.id}>
            <div>
              <h2>{p.name}</h2>
              <span>
                {p.id === "custom"
                  ? "Your selection"
                  : p.sections.flat().length + " modules"}
              </span>
            </div>
            <p>{p.description}</p>
            <div className="preset-module-badges">
              {p.sections.flat().map((id) => {
                const m = catalog.modules.find((m) => m.id === id)!;
                return (
                  <span key={id}>
                    <i>{m.icon}</i>
                    {m.label}
                  </span>
                );
              })}
            </div>
            <Link to={"/?preset=" + encodeURIComponent(p.id) + "#studio"}>
              Explore in the studio →
            </Link>
          </article>
        ))}
      </div>
    </div>
  );
}
function FAQPage() {
  return (
    <div className="marketing-container route-page faq-route">
      <div className="route-intro">
        <p className="eyebrow">FREQUENTLY ASKED QUESTIONS</p>
        <h1>
          A clearer picture.
          <br />
          Before your first prompt.
        </h1>
        <p>How the native app, browser studio and Fastfetch work together.</p>
      </div>
      <FAQ full />
      <Link className="button-primary" to="/docs/getting-started">
        Ready? Get started →
      </Link>
    </div>
  );
}
export default function App() {
  return (
    <BrowserRouter>
      <Site />
    </BrowserRouter>
  );
}
