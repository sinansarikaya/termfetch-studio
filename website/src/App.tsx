import { useEffect, useState } from "react";
import {
  BrowserRouter,
  NavLink,
  Route,
  Routes,
  Link,
  useLocation,
} from "react-router-dom";
import {
  catalog,
  defaults,
  getTheme,
  getLogo,
  getPreset,
  groups,
  colors,
  dots,
  cssColor,
  jsonc,
  download,
  validColor,
} from "./lib/studio";
import type { Settings } from "./lib/studio";
import { Downloads, Docs, Releases } from "./pages";
import { publicOrigin, release } from "./lib/distribution";
import "./App.css";

const screens = [
  "Dashboard",
  "Themes",
  "Presets",
  "Logos",
  "Modules",
  "Settings",
] as const;
type Screen = (typeof screens)[number];
const categoryNames: Record<string, string> = {
  Community: "Top Community",
  Dark: "Dark",
  Light: "Light",
  Pastel: "Pastel",
  PopCulture: "Pop Culture & Gaming",
};
function Brand() {
  return (
    <>
      <img src="/brand/termfetch-symbol.svg" alt="" width="30" height="30" />
      <span>
        TermFetch <b>Studio</b>
      </span>
    </>
  );
}
function Site() {
  const location = useLocation();
  useEffect(() => {
    const labels: Record<string, string> = {
      "/": "Fastfetch customization studio",
      "/downloads": "Downloads",
      "/docs": "Getting started",
      "/docs/getting-started": "Getting started",
      "/releases": "Releases",
    };
    document.title =
      "TermFetch Studio — " + (labels[location.pathname] ?? "Page not found");
    const canonical =
      publicOrigin + (location.pathname === "/" ? "/" : location.pathname);
    document
      .querySelector<HTMLLinkElement>('link[rel="canonical"]')
      ?.setAttribute("href", canonical);
    document
      .querySelector<HTMLMetaElement>('meta[property="og:url"]')
      ?.setAttribute("content", canonical);
    if (location.hash) {
      const target = document.getElementById(location.hash.slice(1));
      target?.scrollIntoView();
      target?.focus({ preventScroll: true });
    } else window.scrollTo(0, 0);
  }, [location.pathname, location.hash]);
  return (
    <>
      <a
        className="skip"
        href="#main"
        onClick={(event) => {
          event.preventDefault();
          document.getElementById("main")?.focus();
        }}
      >
        {" "}
        Skip to content
      </a>
      <header className="site-header">
        <Link to="/" className="brand" aria-label="TermFetch Studio home">
          <Brand />
        </Link>
        <nav aria-label="Main navigation">
          <NavLink to="/" end>
            Studio
          </NavLink>
          <NavLink to="/downloads">Downloads</NavLink>
          <NavLink to="/docs">Docs</NavLink>
          <NavLink to="/releases">Releases</NavLink>
        </nav>
        <a
          className="source-link"
          href="https://github.com/sinansarikaya/termfetch-studio"
        >
          Source ↗
        </a>
      </header>
      <main id="main" tabIndex={-1} key={location.pathname}>
        <Routes>
          <Route path="/" element={<Studio />} />
          <Route path="/downloads" element={<Downloads />} />
          <Route path="/docs" element={<Docs />} />
          <Route path="/docs/getting-started" element={<Docs />} />
          <Route path="/releases" element={<Releases />} />
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
      <footer>
        <Link to="/" className="brand">
          <Brand />
        </Link>
        <p>Built with Rust. Previewed in your browser.</p>
        <nav aria-label="Footer navigation">
          <Link to="/docs">Getting started</Link>
          <Link to="/downloads">Download status</Link>
          <a href="https://github.com/sinansarikaya/termfetch-studio">
            GitHub ↗
          </a>
        </nav>
        <small>
          MIT · Released v{release.version} · Source v{catalog.version}
        </small>
      </footer>
    </>
  );
}
function Select({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <label className="field">
      <span>{label}</span>
      <select
        aria-label={label}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </label>
  );
}
function Studio() {
  const [s, setS] = useState<Settings>(() => structuredClone(defaults));
  const [saved, setSaved] = useState<Settings>(() => structuredClone(defaults));
  const [screen, setScreen] = useState<Screen>("Dashboard");
  const [output, setOutput] = useState("preview");
  const [search, setSearch] = useState("");
  const [category, setCategory] = useState("all");
  const [status, setStatus] = useState(
    "Ready. Your browser preview uses privacy-safe sample specs.",
  );
  const [draft, setDraft] = useState<Settings | null>(null);
  const [recent, setRecent] = useState<string[]>([]);
  const [helpOpen, setHelpOpen] = useState(false);
  const theme = getTheme(s),
    preset = getPreset(s),
    logo = getLogo(s),
    code = jsonc(s);
  const dirty = JSON.stringify(s) !== JSON.stringify(saved);
  const update = (patch: Partial<Settings>) =>
    setS((old) => ({ ...old, ...patch }));
  const active = draft
    ? {
        ...s,
        icon_color: draft.icon_color,
        text_color: draft.text_color,
        group_colors: draft.group_colors,
      }
    : s;
  const changeScreen = (value: Screen) => {
    setScreen(value);
    setSearch("");
  };
  function apply() {
    setSaved(structuredClone(s));
    setRecent((old) =>
      [s.theme, ...old.filter((id) => id !== s.theme)].slice(0, 8),
    );
    try {
      localStorage.setItem("termfetch-browser-export", JSON.stringify(s));
      setStatus(
        "Applied to this browser only. No host files or terminal settings were changed. Export JSONC to use with Fastfetch.",
      );
    } catch {
      setStatus(
        "Applied for this session. Browser storage is unavailable; download JSONC to keep your work.",
      );
    }
  }
  function restore() {
    try {
      const raw = localStorage.getItem("termfetch-browser-export");
      if (!raw) {
        setStatus(
          "No saved browser configuration yet. Choose Apply locally first.",
        );
        return;
      }
      const candidate: unknown = JSON.parse(raw);
      if (!isSettings(candidate)) {
        setStatus(
          "Saved browser data is invalid or from another catalog. Reset or apply a new configuration.",
        );
        return;
      }
      setS(candidate);
      setSaved(structuredClone(candidate));
      setStatus(
        "Restored the last browser configuration. Host files are unchanged.",
      );
    } catch {
      setStatus(
        "Could not read browser storage. You can still customize and export.",
      );
    }
  }
  function toggleModule(id: string) {
    update({
      preset: "custom",
      custom_modules: s.custom_modules.includes(id)
        ? s.custom_modules.filter((m) => m !== id)
        : [...s.custom_modules, id],
    });
  }
  function reorder(id: string, delta: number) {
    const list = [...s.custom_modules],
      i = list.indexOf(id),
      next = i + delta;
    if (i < 0 || next < 0 || next >= list.length) return;
    [list[i], list[next]] = [list[next], list[i]];
    update({ preset: "custom", custom_modules: list });
  }
  async function copy() {
    try {
      await navigator.clipboard.writeText(code);
      setStatus(
        "JSONC copied. Save it as config.jsonc and run Fastfetch with that file.",
      );
    } catch {
      setStatus(
        "Clipboard unavailable. Use Download JSONC or select the generated output manually.",
      );
      setOutput("jsonc");
    }
  }
  function handleStudioKey(event: React.KeyboardEvent<HTMLElement>) {
    const target = event.target as HTMLElement;
    if (
      target.closest('input, select, textarea, [contenteditable="true"]') ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey
    )
      return;
    const key = event.key.toLowerCase();
    if (/^[1-6]$/.test(key)) changeScreen(screens[Number(key) - 1]);
    else if (key === "a" && !draft) apply();
    else if (key === "d") update({ demo_mode: !s.demo_mode });
    else if (key === "o") setOutput(output === "preview" ? "jsonc" : "preview");
    else if (key === "?") setHelpOpen(!helpOpen);
    else if (key === "escape") {
      setHelpOpen(false);
      setDraft(null);
    } else return;
    event.preventDefault();
  }
  const options = (
    <div className="preview-options">
      <Select
        label="Theme"
        value={s.theme}
        onChange={(theme) => update({ theme })}
        options={catalog.themes.map((t) => ({ value: t.id, label: t.name }))}
      />
      <Select
        label="Preset"
        value={s.preset}
        onChange={(preset) => update({ preset })}
        options={catalog.presets.map((p) => ({ value: p.id, label: p.name }))}
      />
      <Select
        label="Logo"
        value={s.logo}
        onChange={(logo) => update({ logo })}
        options={catalog.logos.map((l) => ({ value: l.id, label: l.name }))}
      />
      <Select
        label="Image size"
        value={s.logo_size}
        onChange={(logo_size) => update({ logo_size })}
        options={[
          { value: "small", label: "Small · max 20×14 cells" },
          { value: "medium", label: "Medium · max 36×22 cells" },
          { value: "large", label: "Large · max 52×32 cells" },
        ]}
      />
    </div>
  );
  return (
    <div className="studio-page">
      <section className="intro">
        <div>
          <p className="eyebrow">TERMINAL CUSTOMIZATION / BUILT IN RUST</p>
          <h1>
            Your terminal.
            <br />
            <em>Your signature.</em>
          </h1>
        </div>
        <div className="intro-note">
          <p>
            Shape your Fastfetch setup in a live studio. Pick a palette, build
            your layout, and take the config with you.
          </p>
          <div className="inline-links">
            <Link to="/docs">Get started ↗</Link>
            <Link to="/downloads">Download status →</Link>
          </div>
        </div>
      </section>
      <section
        className="studio"
        aria-label="Interactive TermFetch Studio"
        tabIndex={0}
        onKeyDown={handleStudioKey}
      >
        <div className="studio-title">
          <span>
            <span className="signal" /> termfetch-studio{" "}
            <span className="muted">/ browser edition</span>
          </span>
          <div className="studio-tools">
            <span className="version">SOURCE {catalog.version}</span>
            <button
              onClick={() => setHelpOpen(!helpOpen)}
              aria-expanded={helpOpen}
              aria-controls="studio-shortcuts"
            >
              ? Shortcuts
            </button>
          </div>
        </div>
        {helpOpen && (
          <div
            className="shortcut-help"
            id="studio-shortcuts"
            role="region"
            aria-label="Studio keyboard shortcuts"
          >
            <p>
              <strong>Keyboard studio</strong> · Focus the studio to use
              shortcuts. Text fields keep normal typing.
            </p>
            <p>
              <kbd>1</kbd>–<kbd>6</kbd> Sections · <kbd>a</kbd> Apply locally ·{" "}
              <kbd>d</kbd> Demo export · <kbd>o</kbd> Output · <kbd>?</kbd> Help
              · <kbd>Esc</kbd> Cancel colors / close help
            </p>
            <button onClick={() => setHelpOpen(false)}>Close shortcuts</button>
          </div>
        )}
        <div className="workspace">
          <aside className="sidebar">
            <p className="eyebrow">WORKSPACE</p>
            <nav aria-label="Studio sections">
              {screens.map((name, i) => (
                <button
                  key={name}
                  className={name === screen ? "selected" : ""}
                  aria-current={name === screen ? "page" : undefined}
                  onClick={() => changeScreen(name)}
                >
                  <span className="nav-index">0{i + 1}</span>
                  {name}
                  <span aria-hidden="true">↗</span>
                </button>
              ))}
            </nav>
            <div className="current-config">
              <small>CURRENT CONFIGURATION</small>
              <strong>{theme.name}</strong>
              <span>
                {preset.name} · {logo.name}
              </span>
              <span className={dirty ? "pending" : "muted"}>
                {dirty ? "● Unapplied changes" : "✓ Browser baseline"}
              </span>
            </div>
            <p className="sidebar-note">
              A local simulation.
              <br />
              No host access. No account.
            </p>
          </aside>
          <section className="editor" aria-label={screen}>
            <div className="section-heading">
              <div>
                <p className="eyebrow">STUDIO / {screen.toUpperCase()}</p>
                <h2>{screen === "Dashboard" ? "Make it yours." : screen}</h2>
              </div>
              <span className="section-number">
                {String(screens.indexOf(screen) + 1).padStart(2, "0")}
              </span>
            </div>
            {screen === "Dashboard" && (
              <>
                <p className="muted">
                  Start with a palette. Keep the parts you need.
                </p>
                <div className="quick-actions">
                  <button onClick={() => changeScreen("Themes")}>
                    <strong>Browse themes</strong>
                    <span>{catalog.themes.length} built-in palettes →</span>
                  </button>
                  <button onClick={() => changeScreen("Modules")}>
                    <strong>Build a layout</strong>
                    <span>{catalog.modules.length} modules, your order →</span>
                  </button>
                  <button onClick={() => changeScreen("Logos")}>
                    <strong>Choose a logo</strong>
                    <span>ASCII & image catalog →</span>
                  </button>
                  <button
                    onClick={() => {
                      setOutput("jsonc");
                      setStatus("Generated JSONC is visible in Live Preview.");
                    }}
                  >
                    <strong>Inspect output</strong>
                    <span>Fastfetch JSONC →</span>
                  </button>
                </div>
                <h3>Recent & recommended</h3>
                <div className="recent-themes">
                  {[
                    ...new Set([
                      ...recent,
                      "catppuccin-mocha",
                      "monokai-pro",
                      "tokyo-night",
                      "nord",
                    ]),
                  ]
                    .slice(0, 4)
                    .map((id) => {
                      const t = catalog.themes.find((t) => t.id === id)!;
                      return (
                        <button
                          key={id}
                          onClick={() => update({ theme: id })}
                          aria-pressed={s.theme === id}
                        >
                          <svg viewBox="0 0 160 66" aria-hidden="true">
                            <rect
                              width="160"
                              height="66"
                              rx="5"
                              fill={t.background}
                            />
                            <path
                              d="M14 18h35m-35 10h68m-68 10h52m-52 10h85"
                              stroke={t.primary}
                              strokeWidth="3"
                            />
                            <circle cx="132" cy="23" r="6" fill={t.accent} />
                          </svg>
                          <span>{t.name}</span>
                        </button>
                      );
                    })}
                </div>
                <div className="note">
                  <strong>Sample system</strong>
                  <p>
                    {catalog.demo.os}
                    <br />
                    {catalog.demo.cpu}
                  </p>
                  <small>
                    These are the Rust app's demo values, never a scan of your
                    device.
                  </small>
                </div>
              </>
            )}
            {screen === "Themes" && (
              <>
                <div className="filters">
                  <label className="field">
                    <span>Find a theme</span>
                    <input
                      type="search"
                      value={search}
                      onChange={(e) => setSearch(e.target.value)}
                      placeholder="Search palettes…"
                    />
                  </label>
                  <Select
                    label="Category"
                    value={category}
                    onChange={setCategory}
                    options={[
                      { value: "all", label: "All categories" },
                      ...Object.entries(categoryNames).map(
                        ([value, label]) => ({ value, label }),
                      ),
                    ]}
                  />
                </div>
                <div className="theme-list">
                  {catalog.themes
                    .filter(
                      (t) =>
                        (category === "all" || t.category === category) &&
                        t.name.toLowerCase().includes(search.toLowerCase()),
                    )
                    .map((t) => (
                      <button
                        className="theme-choice"
                        key={t.id}
                        aria-pressed={s.theme === t.id}
                        onClick={() => update({ theme: t.id })}
                      >
                        <svg viewBox="0 0 180 65" aria-hidden="true">
                          <rect width="180" height="65" fill={t.background} />
                          <path
                            d="M12 15h50m-50 12h100m-100 12h73m-73 12h120"
                            stroke={t.primary}
                            strokeWidth="3"
                          />
                          {[t.secondary, t.accent, t.groups.hardware].map(
                            (c, i) => (
                              <circle
                                key={i}
                                cx={133 + i * 15}
                                cy="16"
                                r="4"
                                fill={c}
                              />
                            ),
                          )}
                        </svg>
                        <strong>{t.name}</strong>
                        <small>
                          {categoryNames[t.category]}{" "}
                          {s.theme === t.id ? "· Selected" : ""}
                        </small>
                      </button>
                    ))}
                </div>
                {!catalog.themes.some(
                  (t) =>
                    (category === "all" || t.category === category) &&
                    t.name.toLowerCase().includes(search.toLowerCase()),
                ) && (
                  <p>
                    No matching palettes. Clear the search or choose another
                    category.
                  </p>
                )}
              </>
            )}
            {screen === "Presets" && (
              <div className="preset-list">
                {catalog.presets.map((p) => (
                  <button
                    key={p.id}
                    aria-pressed={s.preset === p.id}
                    onClick={() => update({ preset: p.id })}
                  >
                    <div>
                      <strong>{p.name}</strong>
                      <span>{p.description}</span>
                    </div>
                    <small>
                      {p.id === "custom"
                        ? s.custom_modules.length
                        : p.sections.flat().length}{" "}
                      modules {s.preset === p.id ? "✓" : "→"}
                    </small>
                  </button>
                ))}
              </div>
            )}
            {screen === "Logos" && (
              <>
                <p className="muted">
                  The app's built-in catalog. Image previews use bundled
                  samples; export needs a path on your machine.
                </p>
                <div className="logo-list">
                  {catalog.logos.map((l) => (
                    <button
                      key={l.id}
                      aria-pressed={s.logo === l.id}
                      onClick={() => update({ logo: l.id })}
                    >
                      {l.kind === "image" ? (
                        <img
                          loading="lazy"
                          src={"/preset-logos/" + encodeURIComponent(l.file!)}
                          alt=""
                        />
                      ) : l.kind === "ascii" ? (
                        <pre aria-hidden="true">{l.art}</pre>
                      ) : (
                        <span className="custom-logo-symbol" aria-hidden="true">
                          ＋
                        </span>
                      )}
                      <span>{l.name}</span>
                      <small>
                        {l.kind === "ascii"
                          ? "ASCII"
                          : l.kind === "image"
                            ? "PNG preset"
                            : "Local custom asset"}
                      </small>
                    </button>
                  ))}
                </div>
              </>
            )}
            {screen === "Modules" && (
              <>
                <p className="muted">
                  Changes activate Custom Modules. Enabled modules appear first;
                  use the arrow buttons to reorder with mouse or keyboard.
                </p>
                <button
                  className="secondary"
                  onClick={() =>
                    update({
                      preset: "custom",
                      custom_modules: preset.sections.flat(),
                    })
                  }
                >
                  Use current preset as a starting point
                </button>
                <p className="module-count">
                  {s.custom_modules.length} enabled / {catalog.modules.length}{" "}
                  available
                </p>
                <div className="module-list">
                  {[
                    ...s.custom_modules,
                    ...catalog.modules
                      .map((m) => m.id)
                      .filter((id) => !s.custom_modules.includes(id)),
                  ].map((id) => {
                    const m = catalog.modules.find((m) => m.id === id)!,
                      i = s.custom_modules.indexOf(id),
                      enabled = i >= 0;
                    return (
                      <div key={id} className="module-row">
                        <label>
                          <input
                            type="checkbox"
                            checked={enabled}
                            onChange={() => toggleModule(id)}
                          />
                          <span>
                            <strong>
                              {enabled
                                ? String(i + 1).padStart(2, "0") + " · "
                                : ""}
                              {m.label}
                            </strong>
                            <small>{m.description}</small>
                          </span>
                        </label>
                        <div>
                          <button
                            aria-label={"Move " + m.label + " up"}
                            disabled={!enabled || i === 0}
                            onClick={() => reorder(id, -1)}
                          >
                            ↑
                          </button>
                          <button
                            aria-label={"Move " + m.label + " down"}
                            disabled={
                              !enabled || i === s.custom_modules.length - 1
                            }
                            onClick={() => reorder(id, 1)}
                          >
                            ↓
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>
                {s.custom_modules.length === 0 && (
                  <p className="note">
                    No data modules enabled. The exported layout will contain
                    only its title and color dots.
                  </p>
                )}
              </>
            )}
            {screen === "Settings" && (
              <>
                <h3>Preview & export</h3>
                <label className="toggle-row">
                  <span>
                    <strong>Demo mode</strong>
                    <small>
                      On: fixed privacy-safe app values. Off: Fastfetch detects
                      real values when you run the exported file.
                    </small>
                  </span>
                  <input
                    type="checkbox"
                    checked={s.demo_mode}
                    onChange={(e) => update({ demo_mode: e.target.checked })}
                  />
                </label>
                <p className="note">
                  The browser always shows samples. It cannot detect hardware,
                  change terminal colors, run Fastfetch on exit, perform Doctor
                  checks, or manage host backups. Those features belong to the
                  native app.
                </p>
                <h3>Semantic colors</h3>
                <p className="muted">
                  Group overrides color both keys and values. Inherit follows
                  the global icon and text settings.
                </p>
                {!draft ? (
                  <button
                    className="secondary"
                    onClick={() => setDraft(structuredClone(s))}
                  >
                    Edit colors
                  </button>
                ) : (
                  <div className="color-editor">
                    <p className="pending">
                      Editing colors · live preview, not yet applied to your
                      configuration.
                    </p>
                    {(["icon_color", "text_color", ...groups] as const).map(
                      (key) => {
                        const group = groups.includes(
                          key as (typeof groups)[number],
                        );
                        const value = group
                          ? draft.group_colors[key as (typeof groups)[number]]
                          : draft[key as "icon_color" | "text_color"];
                        return (
                          <div className="color-row" key={key}>
                            <Select
                              label={
                                key === "icon_color"
                                  ? "Global icons"
                                  : key === "text_color"
                                    ? "Global text"
                                    : key.toUpperCase()
                              }
                              value={value}
                              onChange={(v) =>
                                setDraft({
                                  ...draft,
                                  ...(group
                                    ? {
                                        group_colors: {
                                          ...draft.group_colors,
                                          [key]: v,
                                        },
                                      }
                                    : { [key]: v }),
                                })
                              }
                              options={[
                                ...(group
                                  ? [
                                      {
                                        value: "inherit",
                                        label: "Inherit globals",
                                      },
                                    ]
                                  : []),
                                ...catalog.palette,
                                ...(!catalog.palette.some(
                                  (p) => p.value === value,
                                ) && value !== "inherit"
                                  ? [{ value, label: value }]
                                  : []),
                              ]}
                            />
                            <label className="field">
                              <span>Custom hex</span>
                              <input
                                aria-label={"Custom hex for " + key}
                                type="color"
                                value={cssColor(
                                  resolveDraft(value, key, draft),
                                  theme.text,
                                )}
                                onChange={(e) =>
                                  setDraft({
                                    ...draft,
                                    ...(group
                                      ? {
                                          group_colors: {
                                            ...draft.group_colors,
                                            [key]: e.target.value,
                                          },
                                        }
                                      : { [key]: e.target.value }),
                                  })
                                }
                              />
                            </label>
                          </div>
                        );
                      },
                    )}
                    <div className="button-row">
                      <button
                        className="primary"
                        onClick={() => {
                          setS((old) => ({
                            ...old,
                            icon_color: draft.icon_color,
                            text_color: draft.text_color,
                            group_colors: { ...draft.group_colors },
                          }));
                          setDraft(null);
                          setStatus(
                            "Color edits applied to your browser configuration.",
                          );
                        }}
                      >
                        Apply colors
                      </button>
                      <button onClick={() => setDraft(null)}>
                        Cancel colors
                      </button>
                      <button
                        onClick={() =>
                          setDraft({
                            ...draft,
                            icon_color: "theme",
                            text_color: "theme",
                            group_colors: { ...defaults.group_colors },
                          })
                        }
                      >
                        Reset colors
                      </button>
                    </div>
                  </div>
                )}
                <h3>Local session</h3>
                <div className="button-row">
                  <button onClick={restore}>Restore browser save</button>
                  <button
                    onClick={() => {
                      setS(structuredClone(defaults));
                      setDraft(null);
                      setStatus(
                        "Restored Rust app defaults in the preview. Apply locally to replace your browser save.",
                      );
                    }}
                  >
                    Reset all to defaults
                  </button>
                  <button
                    onClick={() => {
                      setS(structuredClone(saved));
                      setDraft(null);
                      setStatus("Unapplied changes cancelled.");
                    }}
                  >
                    Cancel unapplied changes
                  </button>
                </div>
              </>
            )}
          </section>
          <aside className="preview-column" aria-label="Live Preview">
            <div className="preview-heading">
              <h2>Live Preview</h2>
              <span className="sample-label">SAMPLE DATA</span>
            </div>
            <div className="output-tabs" aria-label="Preview output">
              <button
                aria-pressed={output === "preview"}
                onClick={() => setOutput("preview")}
              >
                Terminal preview
              </button>
              <button
                aria-pressed={output === "jsonc"}
                onClick={() => setOutput("jsonc")}
              >
                Generated JSONC
              </button>
            </div>
            {output === "preview" ? (
              <Preview s={active} />
            ) : (
              <pre
                className="json-output"
                tabIndex={0}
                aria-label="Generated Fastfetch JSONC"
              >
                {jsonc(active)}
              </pre>
            )}
            <p className="preview-disclaimer">
              Approximate browser rendering. Real terminal geometry, Nerd Font
              glyphs and image protocols affect the native result.
            </p>
            {options}
            {(logo.kind === "image" || logo.kind === "custom") && (
              <div className="asset-options">
                <label className="field">
                  <span>Image path on your machine (optional)</span>
                  <input
                    value={s.image_path}
                    placeholder="/absolute/path/to/logo.png"
                    onChange={(e) => update({ image_path: e.target.value })}
                  />
                </label>
                <Select
                  label="Image protocol"
                  value={s.image_protocol}
                  onChange={(image_protocol) => update({ image_protocol })}
                  options={["chafa", "kitty-direct", "iterm", "sixel"].map(
                    (value) => ({ value, label: value }),
                  )}
                />
                {logo.kind === "custom" && (
                  <label className="field">
                    <span>Or paste custom ASCII</span>
                    <textarea
                      value={s.custom_art}
                      onChange={(e) => update({ custom_art: e.target.value })}
                      rows={4}
                    />
                  </label>
                )}
                <small>
                  {s.image_path
                    ? "Export uses this path. Size sets maximum width; Fastfetch preserves aspect ratio."
                    : "No local image path: export falls back to automatic distro logo" +
                      (s.custom_art
                        ? " unless custom ASCII is supplied."
                        : ".")}
                </small>
              </div>
            )}
            <button
              className="primary apply"
              onClick={apply}
              disabled={draft !== null}
            >
              Apply locally <span>↗</span>
            </button>
            <p className="local-note">
              Saves in this browser. Never writes to your system.
            </p>
            <div className="button-row export-actions">
              <button disabled={draft !== null} onClick={copy}>
                Copy JSONC
              </button>
              <button
                disabled={draft !== null}
                onClick={() => {
                  download(code, "config.jsonc");
                  setStatus(
                    "JSONC download requested. See Docs for using the file with Fastfetch.",
                  );
                }}
              >
                Download JSONC ↓
              </button>
            </div>
          </aside>
        </div>
        <div className="studio-status" role="status" aria-live="polite">
          <span className="signal" />
          {status}
        </div>
      </section>
      <section className="below-studio">
        <div>
          <p className="eyebrow">FROM PREVIEW TO TERMINAL</p>
          <h2>
            A config you can read.
            <br />A setup you control.
          </h2>
        </div>
        <div>
          <p>
            Export typed Fastfetch settings with the same catalog and color
            resolution used by the Rust app. ASCII logos travel inside your
            config. Image logos use a local path you supply.
          </p>
          <Link to="/docs/getting-started">Use your exported config →</Link>
        </div>
      </section>
    </div>
  );
}
function resolveDraft(value: string, key: string, s: Settings) {
  const t = getTheme(s);
  return value === "theme" || value === "inherit"
    ? groups.includes(key as (typeof groups)[number])
      ? t.groups[key as (typeof groups)[number]]
      : key === "icon_color"
        ? t.groups.os
        : t.text
    : value;
}
function isSettings(value: unknown): value is Settings {
  if (!value || typeof value !== "object") return false;
  const s = value as Settings;
  return (
    catalog.themes.some((t) => t.id === s.theme) &&
    catalog.presets.some((p) => p.id === s.preset) &&
    catalog.logos.some((l) => l.id === s.logo) &&
    ["small", "medium", "large"].includes(s.logo_size) &&
    typeof s.demo_mode === "boolean" &&
    typeof s.apply_terminal_colors === "boolean" &&
    typeof s.run_fastfetch_on_exit === "boolean" &&
    typeof s.ui_follow_theme === "boolean" &&
    Array.isArray(s.recent_themes) &&
    s.recent_themes.length <= 8 &&
    s.recent_themes.every((id) => catalog.themes.some((t) => t.id === id)) &&
    Array.isArray(s.custom_modules) &&
    s.custom_modules.length <= 25 &&
    new Set(s.custom_modules).size === s.custom_modules.length &&
    s.custom_modules.every((id) => catalog.modules.some((m) => m.id === id)) &&
    typeof s.icon_color === "string" &&
    validColor(s.icon_color) &&
    typeof s.text_color === "string" &&
    validColor(s.text_color) &&
    !!s.group_colors &&
    groups.every(
      (g) =>
        typeof s.group_colors[g] === "string" && validColor(s.group_colors[g]),
    ) &&
    typeof s.image_path === "string" &&
    s.image_path.length <= 8192 &&
    typeof s.custom_art === "string" &&
    s.custom_art.length <= 65536 &&
    ["chafa", "kitty-direct", "iterm", "sixel"].includes(s.image_protocol)
  );
}
function Preview({ s }: { s: Settings }) {
  const t = getTheme(s),
    p = getPreset(s),
    l = getLogo(s);
  // Only catalog colors and validated color selections reach CSS, never free-form input.
  const rules =
    ".terminal-preview{background:" +
    t.background +
    ";color:" +
    t.text +
    "}.preview-title,.ascii-art{color:" +
    t.primary +
    "}" +
    groups
      .map((g) => {
        const c = colors(s, g);
        return (
          ".group-" +
          g +
          " .preview-key{color:" +
          cssColor(c.key, t.text) +
          "}.group-" +
          g +
          " .preview-value{color:" +
          cssColor(c.output, t.text) +
          "}"
        );
      })
      .join("") +
    dots(s)
      .map((c, i) => ".dot-" + i + "{color:" + cssColor(c, t.text) + "}")
      .join("");
  const dotRow = (
    <div className="color-dots" aria-label="Theme color dots">
      {dots(s).map((_, i) => (
        <span key={i} className={"dot-" + i}>
          ●{" "}
        </span>
      ))}
    </div>
  );
  return (
    <>
      <style>{rules}</style>
      <div
        className="terminal-preview"
        tabIndex={0}
        aria-label="Fastfetch preview using sample specs"
      >
        <div className={"logo-pane size-" + s.logo_size}>
          {l.kind === "image" ? (
            <img
              src={"/preset-logos/" + encodeURIComponent(l.file!)}
              alt={l.name}
            />
          ) : l.kind === "custom" ? (
            <pre className="ascii-art">
              {s.custom_art ||
                "[Your local image]\nBrowser cannot read this path"}
            </pre>
          ) : (
            <pre className="ascii-art">{l.art}</pre>
          )}
        </div>
        <div className="preview-data">
          <div className="preview-title">{catalog.demo.user_host}</div>
          {p.color_dots && dotRow}
          {p.sections.map((section, i) => (
            <div className="preview-section" key={i}>
              {section.map((id) => {
                const m = catalog.modules.find((m) => m.id === id)!;
                return (
                  <div className={"preview-line group-" + m.group} key={id}>
                    <span className="preview-key" title={m.label}>
                      {m.label}
                    </span>
                    <span aria-hidden="true"> ➜ </span>
                    <span className="preview-value">{m.demo}</span>
                  </div>
                );
              })}
            </div>
          ))}
          {p.color_dots && dotRow}
        </div>
      </div>
    </>
  );
}
export default function App() {
  return (
    <BrowserRouter>
      <Site />
    </BrowserRouter>
  );
}
