import { useState } from "react";
import type { CSSProperties, KeyboardEvent, ReactNode } from "react";
import {
  catalog,
  defaults,
  getTheme,
  getLogo,
  getPreset,
  groups,
  colors,
  cssColor,
  jsonc,
  download,
  validColor,
} from "./lib/studio";
import type { Settings, Theme } from "./lib/studio";
import { LOOKS, settingsForLook } from "./lib/looks";

const MAX_PRESETS = 40;
const nativeScreens = [
  "Dashboard",
  "Themes",
  "Presets",
  "Logos",
  "Modules",
  "Settings",
] as const;
type Screen = (typeof nativeScreens)[number];
const navIcons = ["󰕮", "󰏘", "󰉉", "󰋔", "󰅩", "󰒓"];
/// Studio chrome colours of the native app (`tui/palette.rs`).
const STUDIO = {
  ok: "#4ade80",
  warn: "#fbbf24",
  error: "#f87171",
  info: "#60a5fa",
  special: "#c084fc",
  orange: "#fb923c",
  accent2: "#f472b6",
  link: "#67e8f9",
  muted: "#a6aec4",
};
/// Fill-up themes of the native Recent Themes row.
const RECENT_FAVOURITES = [
  "monokai-pro",
  "tokyo-night",
  "catppuccin-mocha",
  "nord",
  "dracula",
  "rose-pine",
];
const tools = [
  {
    icon: "\u{f0147}",
    color: STUDIO.warn,
    title: "System Diagnostics",
    desc: "Check dependencies and system info",
  },
  {
    icon: "\u{f018d}",
    color: STUDIO.info,
    title: "Shell Integration",
    desc: "Configure your shell (zsh, bash, fish)",
  },
  {
    icon: "\u{f01bc}",
    color: STUDIO.special,
    title: "Backup & Restore",
    desc: "Manage your configurations",
  },
  {
    icon: "\u{f0552}",
    color: STUDIO.link,
    title: "Import / Export",
    desc: "Share themes and presets",
  },
];
/// Screen-specific footer keys, as in the native `Screen::extra_keys`.
const footerKeys: Record<Screen, [string, string][]> = {
  Dashboard: [["1-4", "Quick Action"]],
  Themes: [
    ["c", "Category"],
    ["/", "Search"],
    ["a", "Apply"],
  ],
  Presets: [
    ["Enter", "Use"],
    ["d", "Copy"],
    ["r", "Rename"],
    ["x", "Delete"],
  ],
  Logos: [
    ["s", "Size"],
    ["a", "Apply"],
  ],
  Modules: [
    ["Space", "Toggle"],
    ["J/K", "Reorder"],
  ],
  Settings: [["←→", "Change"]],
};
const subtitles: Record<Screen, string> = {
  Dashboard: "Customize your terminal, your way.",
  Themes: "Browse palettes. The live preview follows your selection.",
  Presets: "Choose which modules fastfetch prints and how they are grouped.",
  Logos: "Choose your signature. ASCII art, image logos and custom artwork.",
  Modules: "Choose the information you need. Arrange it in your own order.",
  Settings: "Fine-tune semantic colors, image size and privacy preferences.",
};
/// The theme's signature colours, as the native app shows them: primary,
/// secondary, accent, then the five module groups.
export function ThemeSwatches({ theme }: { theme: Theme }) {
  return (
    <span className="theme-swatches" aria-label={`${theme.name} palette`}>
      {[
        theme.primary,
        theme.secondary,
        theme.accent,
        ...Object.values(theme.groups),
      ].map((color, i) => (
        <i key={i} style={{ backgroundColor: color }} />
      ))}
    </span>
  );
}
/// Distinct colours a theme defines, like the native `theme_color_count`.
function themeColorCount(theme: Theme) {
  return new Set(
    [
      theme.background,
      theme.text,
      theme.primary,
      theme.secondary,
      theme.accent,
      ...Object.values(theme.groups),
    ].map((c) => c.toLowerCase()),
  ).size;
}
/// The native app's theme art: a mini fastfetch in the theme's own colours.
/// Compact (sidebar) keeps only the coloured group lines.
export function ThemeArt({
  theme,
  compact = false,
}: {
  theme: Theme;
  compact?: boolean;
}) {
  const groupColors = Object.values(theme.groups);
  return (
    <div
      className={"theme-art" + (compact ? " compact" : "")}
      style={{ background: theme.background, color: theme.text }}
      aria-hidden="true"
    >
      <div className="art-logo" style={{ color: theme.primary }}>
        {"   /\\\n  /  \\\n / /\\ \\\n/_/  \\_\\"}
      </div>
      <div className="art-lines">
        <b style={{ color: theme.primary }}>cyberdeck@studio</b>
        {groupColors.map((color, i) => (
          <span key={i} style={{ color }}>
            <em>{compact ? "├→" : "├ ▪ → "}</em>
            <i
              style={{
                backgroundColor: color,
                width: `${[82, 62, 94, 72, 48][i]}%`,
              }}
            />
          </span>
        ))}
      </div>
    </div>
  );
}
/// A theme card exactly as in the native Themes grid and Recent Themes:
/// art, optional badge, name row and the palette dots.
function ThemeCard({
  theme,
  selected,
  badge,
  onSelect,
}: {
  theme: Theme;
  selected: boolean;
  badge?: string;
  onSelect: () => void;
}) {
  return (
    <button
      className="native-theme-card"
      onClick={onSelect}
      aria-pressed={selected}
    >
      <ThemeArt theme={theme} />
      {badge && <span className="native-theme-badge">{badge}</span>}
      <span className="native-theme-name">{theme.name}</span>
      <ThemeSwatches theme={theme} />
    </button>
  );
}
function Panel({
  title,
  children,
  className = "",
}: {
  title: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <fieldset className={"native-panel " + className}>
      <legend>{title}</legend>
      {children}
    </fieldset>
  );
}
function validSettings(value: unknown): value is Settings {
  if (!value || typeof value !== "object") return false;
  const s = value as Settings;
  if (
    s.preset_library !== undefined &&
    (!Array.isArray(s.preset_library) ||
      s.preset_library.length < 1 ||
      s.preset_library.length > MAX_PRESETS ||
      (s.preset_library.length === MAX_PRESETS &&
        !s.preset_library.some((p) => p?.id === "custom")) ||
      new Set(s.preset_library.map((p) => p.id)).size !==
        s.preset_library.length ||
      s.preset_library.some(
        (p) =>
          !p ||
          typeof p.id !== "string" ||
          typeof p.name !== "string" ||
          !p.name.trim() ||
          p.name.length > 100 ||
          typeof p.description !== "string" ||
          typeof p.color_dots !== "boolean" ||
          !Array.isArray(p.sections) ||
          p.sections.length > 25 ||
          p.sections.some((section) => !Array.isArray(section)) ||
          p.sections.flat().length > 25 ||
          p.sections
            .flat()
            .some((id) => !catalog.modules.some((m) => m.id === id)),
      ))
  )
    return false;
  return (
    catalog.themes.some((t) => t.id === s.theme) &&
    (s.preset_library ?? catalog.presets).some((p) => p.id === s.preset) &&
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
export function NativePreview({
  settings,
  mini = false,
}: {
  settings: Settings;
  mini?: boolean;
}) {
  const theme = getTheme(settings),
    preset = getPreset(settings),
    logo = getLogo(settings);
  const style = {
    backgroundColor: theme.background,
    color: theme.text,
    "--preview-primary": theme.primary,
  } as CSSProperties;
  return (
    <div
      className={"native-preview-output " + (mini ? "mini-preview" : "")}
      style={style}
      tabIndex={mini ? undefined : 0}
      aria-label="Fastfetch preview with sample system information"
    >
      <div className={"native-logo size-" + settings.logo_size}>
        {logo.kind === "image" ? (
          <img
            src={"/preset-logos/" + encodeURIComponent(logo.file!)}
            alt={logo.name}
            width="200"
            height="220"
          />
        ) : (
          <pre>
            {logo.kind === "custom"
              ? settings.custom_art || "[Your image]"
              : logo.art}
          </pre>
        )}
      </div>
      <div className="native-preview-data">
        <div className="native-user-title">{catalog.demo.user_host}</div>
        {preset.color_dots && <ThemeSwatches theme={theme} />}
        {preset.sections.map((section, i) => (
          <div
            className="native-info-group"
            key={i}
            data-group={catalog.modules.find((m) => m.id === section[0])?.group}
          >
            {section.map((id) => {
              const module = catalog.modules.find((m) => m.id === id)!,
                palette = colors(settings, module.group);
              return (
                <div
                  className="native-info-line"
                  key={id}
                  data-module={id}
                  aria-label={module.label}
                  style={{ color: cssColor(palette.output, theme.text) }}
                >
                  <span
                    className="tree-key"
                    style={{ color: cssColor(palette.key, theme.text) }}
                  >
                    ├
                  </span>
                  <span
                    className="module-glyph"
                    style={{ color: cssColor(palette.key, theme.text) }}
                    title={module.label}
                  >
                    {module.icon}
                  </span>
                  <span className="module-arrow"> → </span>
                  <span className="module-value">{module.demo}</span>
                </div>
              );
            })}
          </div>
        ))}
        {preset.color_dots && <ThemeSwatches theme={theme} />}
      </div>
    </div>
  );
}
export function NativeStudio({
  initial = settingsForLook(LOOKS[0]),
  initialScreen = "Presets",
}: {
  initial?: Settings;
  initialScreen?: Screen;
}) {
  const [settings, setSettings] = useState<Settings>(() =>
    structuredClone(initial),
  );
  const [saved, setSaved] = useState<Settings>(() => structuredClone(initial));
  const [screen, setScreen] = useState<Screen>(initialScreen);
  const [output, setOutput] = useState("preview");
  const [draft, setDraft] = useState<Settings | null>(null);
  const [search, setSearch] = useState("");
  const [category, setCategory] = useState("all");
  const [presetDialog, setPresetDialog] = useState<"rename" | "delete" | null>(
      null,
    ),
    [presetInput, setPresetInput] = useState("");
  const [status, setStatus] = useState("Ready · preview uses sample data");
  const [recentOffset, setRecentOffset] = useState(0);
  const [help, setHelp] = useState(false),
    [expanded, setExpanded] = useState(false),
    [previewExpanded, setPreviewExpanded] = useState(false);
  const theme = getTheme(settings),
    preset = getPreset(settings),
    logo = getLogo(settings);
  const presets = settings.preset_library ?? catalog.presets;
  const presetCopyLimit = presets.some((item) => item.id === "custom")
    ? MAX_PRESETS
    : MAX_PRESETS - 1;
  const canCopyPreset = presets.length < presetCopyLimit;
  const active = draft
    ? {
        ...settings,
        icon_color: draft.icon_color,
        text_color: draft.text_color,
        group_colors: draft.group_colors,
      }
    : settings;
  const update = (patch: Partial<Settings>) => {
    const library = patch.preset_library ?? settings.preset_library;
    if (
      (patch.preset ?? settings.preset) === "custom" &&
      library &&
      !library.some((item) => item.id === "custom") &&
      library.length >= MAX_PRESETS
    ) {
      setStatus(
        "Preset library full · delete a preset before using Custom Modules",
      );
      return;
    }
    setSettings((old) => {
      const next = { ...old, ...patch };
      // Module Builder must remain usable after an editable library deletes its
      // Custom entry. Restore the definition together with its activation so
      // selection, dropdown options and exported sections stay consistent.
      if (
        next.preset === "custom" &&
        next.preset_library &&
        !next.preset_library.some((item) => item.id === "custom")
      ) {
        if (next.preset_library.length >= MAX_PRESETS) return old;
        next.preset_library = [
          ...next.preset_library,
          structuredClone(
            catalog.presets.find((item) => item.id === "custom")!,
          ),
        ];
      }
      return next;
    });
  };
  const dirty = JSON.stringify(settings) !== JSON.stringify(saved);
  // Same order as the native Recent Themes row: applied, recent, favourites.
  const recentThemes = [
    saved.theme,
    ...saved.recent_themes,
    ...RECENT_FAVOURITES,
  ]
    .filter((id, i, ids) => ids.indexOf(id) === i)
    .map((id) => catalog.themes.find((t) => t.id === id))
    .filter((t): t is Theme => !!t)
    .slice(0, 8);
  const quickActions = [
    {
      icon: "\u{f03d8}",
      color: STUDIO.special,
      title: "Edit Theme",
      desc: "Browse & edit palette",
      run: () => setScreen("Themes"),
    },
    {
      icon: "\u{f0249}",
      color: STUDIO.info,
      title: "Manage Presets",
      desc: "Organize module sets",
      run: () => setScreen("Presets"),
    },
    {
      icon: "\u{f02d4}",
      color: STUDIO.orange,
      title: "Change Logo",
      desc: "ASCII & image logos",
      run: () => setScreen("Logos"),
    },
    {
      icon: "\u{f040a}",
      color: STUDIO.ok,
      title: "Apply & Preview",
      desc: "Live terminal preview",
      run: () => apply(),
    },
  ];
  const demo = (id: string) =>
    catalog.modules.find((m) => m.id === id)?.demo ?? "";
  const overview: [string, string, string, string][] = [
    ["\u{f08c7}", STUDIO.info, "OS", demo("os")],
    ["\u{f0322}", STUDIO.error, "Host", demo("host")],
    ["\u{f061a}", STUDIO.orange, "Kernel", demo("kernel")],
    ["\u{f0954}", STUDIO.warn, "Uptime", demo("uptime")],
    ["\u{f07b7}", STUDIO.link, "Terminal", demo("terminal")],
    ["\u{f048d}", STUDIO.special, "Shell", demo("shell")],
    ["󰻠", STUDIO.info, "CPU", demo("cpu")],
    ["󰢮", STUDIO.special, "GPU", demo("gpu")],
    ["󰍛", STUDIO.ok, "Memory", demo("memory")],
    ["󰍹", STUDIO.orange, "Display", demo("display")],
  ];
  function apply() {
    if (!validSettings(settings)) {
      setStatus(
        "Cannot apply an invalid preset library · remove a preset or reset the setup",
      );
      return;
    }
    const next = {
      ...settings,
      recent_themes: [
        settings.theme,
        ...settings.recent_themes.filter((id) => id !== settings.theme),
      ].slice(0, 8),
    };
    setSettings(next);
    setSaved(structuredClone(next));
    try {
      localStorage.setItem("termfetch-browser-export", JSON.stringify(next));
      setStatus("✓ Applied in this browser · host files unchanged");
    } catch {
      setStatus("✓ Applied for this session · storage unavailable");
    }
  }
  function restore() {
    try {
      const raw = localStorage.getItem("termfetch-browser-export");
      if (!raw) {
        setStatus("No saved browser setup yet");
        return;
      }
      const value: unknown = JSON.parse(raw);
      if (!validSettings(value)) {
        setStatus("Saved browser configuration is invalid");
        return;
      }
      setSettings(value);
      setSaved(structuredClone(value));
      setDraft(null);
      const library = value.preset_library ?? catalog.presets;
      const copyLimit = library.some((item) => item.id === "custom")
        ? MAX_PRESETS
        : MAX_PRESETS - 1;
      setStatus(
        library.length >= copyLimit
          ? "✓ Restored browser save · preset library capacity reached"
          : "✓ Restored browser save",
      );
    } catch {
      setStatus("Could not read browser save");
    }
  }
  function copyPreset() {
    if (!canCopyPreset) {
      setStatus(
        "Preset library capacity reached · delete a preset before copying; a slot is reserved for Custom Modules",
      );
      return;
    }
    let n = 1;
    while (presets.some((p) => p.id === `user-${n}`)) n++;
    const copied = {
      ...structuredClone(preset),
      id: `user-${n}`,
      name: `${preset.name} copy`.slice(0, 100),
    };
    update({ preset: copied.id, preset_library: [...presets, copied] });
    setStatus(
      presets.length + 1 >= presetCopyLimit
        ? "✓ Preset copied · preset library capacity reached"
        : "✓ Preset copied · edit or rename your copy",
    );
  }
  function savePresetDialog() {
    if (presetDialog === "rename" && presetInput.trim()) {
      update({
        preset_library: presets.map((p) =>
          p.id === preset.id
            ? { ...p, name: presetInput.trim().slice(0, 100) }
            : p,
        ),
      });
      setPresetDialog(null);
      setStatus("✓ Preset renamed");
    } else if (
      presetDialog === "delete" &&
      presetInput.toLowerCase() === "yes" &&
      presets.length > 1
    ) {
      const remaining = presets.filter((p) => p.id !== preset.id);
      update({ preset: remaining[0].id, preset_library: remaining });
      setPresetDialog(null);
      setStatus("✓ Preset deleted from this browser setup");
    }
  }
  function toggle(id: string) {
    update({
      preset: "custom",
      custom_modules: settings.custom_modules.includes(id)
        ? settings.custom_modules.filter((item) => item !== id)
        : [...settings.custom_modules, id],
    });
  }
  function reorder(id: string, change: number) {
    const list = [...settings.custom_modules],
      i = list.indexOf(id),
      next = i + change;
    if (i < 0 || next < 0 || next >= list.length) return;
    [list[i], list[next]] = [list[next], list[i]];
    update({ preset: "custom", custom_modules: list });
  }
  async function copy() {
    try {
      await navigator.clipboard.writeText(jsonc(settings));
      setStatus("✓ JSONC copied · save it as config.jsonc");
    } catch {
      setOutput("jsonc");
      setStatus("Clipboard unavailable · download JSONC instead");
    }
  }
  function onKey(event: KeyboardEvent<HTMLElement>) {
    if (
      (event.target as HTMLElement).closest(
        "input, select, textarea, [contenteditable=true]",
      ) ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey
    )
      return;
    const key = event.key.toLowerCase();
    if (/^[1-6]$/.test(key)) setScreen(nativeScreens[Number(key) - 1]);
    else if (key === "a" && !draft) apply();
    else if (
      key === "d" &&
      screen === "Presets" &&
      (event.target as HTMLElement).closest(".native-stage")
    )
      copyPreset();
    else if (key === "r" && screen === "Presets") {
      setPresetInput(preset.name);
      setPresetDialog("rename");
    } else if (key === "x" && screen === "Presets" && presets.length > 1) {
      setPresetInput("");
      setPresetDialog("delete");
    } else if (key === "o")
      setOutput(output === "preview" ? "jsonc" : "preview");
    else if (key === "s")
      update({
        logo_size:
          settings.logo_size === "small"
            ? "medium"
            : settings.logo_size === "medium"
              ? "large"
              : "small",
      });
    else if (key === "f") setPreviewExpanded(!previewExpanded);
    else if (key === "?") setHelp(!help);
    else if (key === "escape") {
      setHelp(false);
      setDraft(null);
      setExpanded(false);
      setPreviewExpanded(false);
      setPresetDialog(null);
    } else return;
    event.preventDefault();
  }
  const filteredThemes = catalog.themes.filter(
    (t) =>
      (category === "all" || t.category === category) &&
      t.name.toLowerCase().includes(search.toLowerCase()),
  );
  return (
    <section
      className={"studio-shell " + (expanded ? "studio-expanded" : "")}
      id="studio"
      aria-label="Interactive TermFetch Studio"
    >
      <div className="demo-toolbar">
        <div>
          <span className="live-dot" /> LIVE STUDIO{" "}
          <span className="toolbar-description">
            Explore the real app’s layouts, palettes and modules.
          </span>
        </div>
        <button onClick={() => setExpanded(!expanded)} aria-pressed={expanded}>
          {expanded ? "↙ Close expanded studio" : "⛶ Expand studio"}
        </button>
      </div>
      <div
        className="studio-pan"
        role="region"
        aria-label="Terminal application viewport"
      >
        <div
          className={
            "native-window " + (previewExpanded ? "preview-expanded" : "")
          }
          tabIndex={0}
          onKeyDown={onKey}
        >
          <header className="native-header">
            <span className="native-badge">TF·</span>
            <strong>TermFetch Studio</strong>
            <span className="native-version">v{catalog.version}</span>
            <span className="native-tagline">
              Customize your terminal, your way.
            </span>
            <div className="native-header-status">
              {settings.demo_mode && (
                <span className="native-preview-pill">◉ PREVIEW</span>
              )}
              <span className="native-ok">● Fastfetch: OK (2.70.0)</span>
              <span>│</span>
              <span>󰣇 Arch Linux</span>
              <span>│</span>
              <span>󰅐 09:47</span>
            </div>
          </header>
          <div className="native-body">
            <aside className="native-sidebar">
              <nav aria-label="Studio sections">
                {nativeScreens.map((name, i) => (
                  <button
                    key={name}
                    onClick={() => setScreen(name)}
                    className={screen === name ? "selected" : ""}
                    aria-current={screen === name ? "page" : undefined}
                  >
                    <span>{navIcons[i]}</span>
                    {name}
                    <kbd>{i + 1}</kbd>
                  </button>
                ))}
              </nav>
              <div className="native-context">
                <div>
                  <h3>CURRENT THEME</h3>
                  <div className="native-theme-context">
                    <ThemeArt theme={theme} compact />
                    <span>
                      {theme.name}
                      <small>{themeColorCount(theme)} colors</small>
                    </span>
                  </div>
                  <ThemeSwatches theme={theme} />
                </div>
                <div>
                  <h3>CURRENT PRESET</h3>
                  <p>
                    <span className="native-ok">󰉉</span> {preset.name}
                  </p>
                  <small>OS + Desktop + Hardware</small>
                </div>
                <div>
                  <h3>CONFIG PATH</h3>
                  <p className="native-config-path">
                    <span>󰉋</span> ~/.config/fastfetch
                  </p>
                </div>
              </div>
            </aside>
            <section className="native-stage" aria-label={screen}>
              {screen === "Dashboard" ? (
                <div className="native-stage-heading">
                  <h2>
                    👋 Welcome to{" "}
                    <span
                      className="native-brand-name"
                      style={{ color: STUDIO.accent2 }}
                    >
                      TermFetch Studio
                    </span>
                    !
                  </h2>
                  <p>
                    {theme.name} · {preset.name} · fastfetch 2.70.0
                  </p>
                </div>
              ) : (
                <div className="native-stage-heading">
                  <h2>
                    <span>{navIcons[nativeScreens.indexOf(screen)]}</span>{" "}
                    {screen}
                  </h2>
                  <p>{subtitles[screen]}</p>
                </div>
              )}
              {screen === "Presets" && (
                <>
                  <Panel
                    title={
                      <>
                        Presets · Enter use ·{" "}
                        <button
                          aria-label="Copy preset"
                          onClick={copyPreset}
                          disabled={!canCopyPreset}
                          title={
                            canCopyPreset
                              ? "Duplicate this layout"
                              : "Preset capacity reached (40 total, including reserved Custom Modules slot)"
                          }
                        >
                          d copy
                        </button>{" "}
                        ·{" "}
                        <button
                          aria-label="Rename preset"
                          onClick={() => {
                            setPresetInput(preset.name);
                            setPresetDialog("rename");
                          }}
                        >
                          r rename
                        </button>{" "}
                        ·{" "}
                        <button
                          aria-label="Delete preset"
                          disabled={presets.length <= 1}
                          onClick={() => {
                            setPresetInput("");
                            setPresetDialog("delete");
                          }}
                        >
                          x delete
                        </button>
                      </>
                    }
                    className="native-preset-list"
                  >
                    {presets.map((p) => (
                      <button
                        key={p.id}
                        onClick={() => update({ preset: p.id })}
                        aria-pressed={settings.preset === p.id}
                      >
                        <strong>{p.name}</strong>
                        <span>
                          {p.id === "custom"
                            ? settings.custom_modules.length
                            : p.sections.flat().length}{" "}
                          modules
                        </span>
                        {saved.preset === p.id && <em>✓ applied</em>}
                      </button>
                    ))}
                  </Panel>
                  <Panel
                    title={`${preset.name} · ${preset.sections.flat().length} modules`}
                    className="native-preset-details"
                  >
                    <p>{preset.description}</p>
                    <h3>Categorized Modules:</h3>
                    {preset.sections.map((section, i) => (
                      <p key={i} className="native-category-line">
                        {section.map((id) => {
                          const m = catalog.modules.find((m) => m.id === id)!;
                          return (
                            <span key={id}>
                              <i
                                style={{
                                  color:
                                    theme.groups[
                                      m.group as keyof typeof theme.groups
                                    ],
                                }}
                              >
                                {m.icon}
                              </i>{" "}
                              {m.label}{" "}
                            </span>
                          );
                        })}
                      </p>
                    ))}
                    <p className="native-dim">
                      Select a preset to use its module order. Create your own
                      layout in Modules (5).
                    </p>
                  </Panel>
                </>
              )}
              {screen === "Dashboard" && (
                <div className="native-dashboard">
                  <section className="native-card">
                    <header>
                      <span style={{ color: STUDIO.warn }}>⚡</span> Quick
                      Actions
                      <small>Get started with the most common tasks.</small>
                    </header>
                    <div className="native-quick-actions">
                      {quickActions.map((q, i) => (
                        <button key={q.title} onClick={q.run}>
                          <span>
                            <i style={{ color: q.color }}>{q.icon}</i>
                            <kbd>{i + 1}</kbd>
                          </span>
                          <b>{q.title}</b>
                          <small>{q.desc}</small>
                        </button>
                      ))}
                    </div>
                  </section>
                  <section className="native-card">
                    <header>
                      <span style={{ color: STUDIO.accent2 }}>
                        {"\u{f051a}"}
                      </span>{" "}
                      Recent Themes
                      <small>
                        Quickly switch between your favorite themes.
                      </small>
                      <button
                        className="native-view-all"
                        onClick={() => setScreen("Themes")}
                      >
                        View All →
                      </button>
                    </header>
                    <div className="native-recent">
                      {recentThemes
                        .slice(recentOffset, recentOffset + 2)
                        .map((t) => (
                          <ThemeCard
                            key={t.id}
                            theme={t}
                            selected={settings.theme === t.id}
                            badge={settings.theme === t.id ? "✓" : undefined}
                            onSelect={() => update({ theme: t.id })}
                          />
                        ))}
                      {recentThemes.length > 2 && (
                        <button
                          className="native-recent-scroll"
                          aria-label="More recent themes"
                          onClick={() =>
                            setRecentOffset(
                              recentOffset + 2 < recentThemes.length
                                ? recentOffset + 1
                                : 0,
                            )
                          }
                        >
                          {recentOffset + 2 < recentThemes.length ? "›" : "‹"}
                        </button>
                      )}
                    </div>
                  </section>
                  <div className="native-dashboard-bottom">
                    <section className="native-card">
                      <header>
                        <span style={{ color: STUDIO.info }}>
                          {"\u{f02fc}"}
                        </span>{" "}
                        System Overview
                      </header>
                      <dl className="native-overview">
                        {overview.map(([icon, color, label, value]) => (
                          <div key={label}>
                            <dt>
                              <i style={{ color }}>{icon}</i> {label}
                            </dt>
                            <dd>{value}</dd>
                          </div>
                        ))}
                      </dl>
                    </section>
                    <section className="native-card">
                      <header>
                        <span style={{ color: STUDIO.muted }}>
                          {"\u{f0ad}"}
                        </span>{" "}
                        Tools & Utilities
                      </header>
                      <div className="native-tools">
                        {tools.map((t) => (
                          <button
                            key={t.title}
                            onClick={() =>
                              setStatus(`${t.title} runs in the native app`)
                            }
                          >
                            <i style={{ color: t.color }}>{t.icon}</i>
                            <span>
                              <b>{t.title}</b>
                              <small>{t.desc}</small>
                            </span>
                            ›
                          </button>
                        ))}
                      </div>
                    </section>
                  </div>
                </div>
              )}
              {screen === "Themes" && (
                <Panel
                  title={`Themes · ${filteredThemes.length} results`}
                  className="native-theme-panel"
                >
                  <div className="native-filters">
                    <select
                      aria-label="Theme category"
                      value={category}
                      onChange={(e) => setCategory(e.target.value)}
                    >
                      <option value="all">All categories</option>
                      {[
                        "Community",
                        "Dark",
                        "Light",
                        "Pastel",
                        "PopCulture",
                      ].map((c) => (
                        <option key={c}>{c}</option>
                      ))}
                    </select>
                    <input
                      aria-label="Search themes"
                      type="search"
                      value={search}
                      onChange={(e) => setSearch(e.target.value)}
                      placeholder="/ Search themes"
                    />
                  </div>
                  <div className="native-theme-grid">
                    {filteredThemes.map((t) => (
                      <ThemeCard
                        key={t.id}
                        theme={t}
                        selected={settings.theme === t.id}
                        badge={saved.theme === t.id ? "✓ applied" : undefined}
                        onSelect={() => update({ theme: t.id })}
                      />
                    ))}
                  </div>
                  {!filteredThemes.length && <p>No matching themes.</p>}
                </Panel>
              )}
              {screen === "Logos" && (
                <Panel
                  title={`Logo Library · ${catalog.logos.length} choices`}
                  className="native-logo-panel"
                >
                  <div className="native-logo-grid">
                    {catalog.logos.map((item) => (
                      <button
                        key={item.id}
                        aria-pressed={settings.logo === item.id}
                        onClick={() => update({ logo: item.id })}
                      >
                        {item.kind === "image" ? (
                          <img
                            loading="lazy"
                            src={
                              "/preset-logos/" + encodeURIComponent(item.file!)
                            }
                            alt=""
                          />
                        ) : (
                          <pre>{item.art || "＋"}</pre>
                        )}
                        <span>{item.name}</span>
                      </button>
                    ))}
                  </div>
                  <label className="native-inline-field">
                    Image size
                    <select
                      aria-label="Image size"
                      value={settings.logo_size}
                      onChange={(e) => update({ logo_size: e.target.value })}
                    >
                      {["small", "medium", "large"].map((size) => (
                        <option key={size}>{size}</option>
                      ))}
                    </select>
                  </label>
                </Panel>
              )}
              {screen === "Modules" && (
                <>
                  <Panel
                    title={`󰅩 Module Builder · ${settings.custom_modules.length} enabled`}
                    className="native-module-panel"
                  >
                    <button
                      className="native-text-button"
                      onClick={() =>
                        update({
                          preset: "custom",
                          custom_modules: preset.sections.flat(),
                        })
                      }
                    >
                      Use current preset as a starting point
                    </button>
                    {[
                      ...settings.custom_modules,
                      ...catalog.modules
                        .map((m) => m.id)
                        .filter((id) => !settings.custom_modules.includes(id)),
                    ].map((id) => {
                      const m = catalog.modules.find((m) => m.id === id)!,
                        i = settings.custom_modules.indexOf(id);
                      return (
                        <div className="native-module-row" key={id}>
                          <label>
                            <input
                              type="checkbox"
                              checked={i >= 0}
                              onChange={() => toggle(id)}
                            />
                            <span
                              className="native-module-icon"
                              style={{
                                color:
                                  theme.groups[
                                    m.group as keyof typeof theme.groups
                                  ],
                              }}
                            >
                              {m.icon}
                            </span>
                            <span>
                              {i >= 0 ? `${i + 1}. ` : ""}
                              {m.label}
                            </span>
                          </label>
                          <button
                            aria-label={`Move ${m.label} up`}
                            disabled={i <= 0}
                            onClick={() => reorder(id, -1)}
                          >
                            ↑
                          </button>
                          <button
                            aria-label={`Move ${m.label} down`}
                            disabled={
                              i < 0 || i === settings.custom_modules.length - 1
                            }
                            onClick={() => reorder(id, 1)}
                          >
                            ↓
                          </button>
                        </div>
                      );
                    })}
                  </Panel>
                  <Panel title="Your layout, your order">
                    <p>
                      Space toggles a focused module. Arrow buttons reorder
                      enabled modules. Changes activate Custom Modules.
                    </p>
                  </Panel>
                </>
              )}
              {screen === "Settings" && (
                <Panel
                  title="Settings · colors & preferences"
                  className="native-settings-panel"
                >
                  <label className="native-setting-row">
                    <span>◉ Demo mode</span>
                    <input
                      type="checkbox"
                      aria-label="Demo mode"
                      checked={settings.demo_mode}
                      onChange={(e) => update({ demo_mode: e.target.checked })}
                    />
                  </label>
                  <p className="native-dim">
                    The browser preview always uses samples. Demo off exports
                    real detection for Fastfetch.
                  </p>
                  <label className="native-setting-row">
                    <span>Image size</span>
                    <select
                      aria-label="Image size"
                      value={settings.logo_size}
                      onChange={(e) => update({ logo_size: e.target.value })}
                    >
                      {["small", "medium", "large"].map((size) => (
                        <option key={size}>{size}</option>
                      ))}
                    </select>
                  </label>
                  <h3>Semantic colors</h3>
                  {!draft ? (
                    <button onClick={() => setDraft(structuredClone(settings))}>
                      Edit colors
                    </button>
                  ) : (
                    <div className="native-color-editor">
                      {(["icon_color", "text_color", ...groups] as const).map(
                        (key) => {
                          const group = groups.includes(
                              key as (typeof groups)[number],
                            ),
                            value = group
                              ? draft.group_colors[
                                  key as (typeof groups)[number]
                                ]
                              : draft[key as "icon_color" | "text_color"];
                          const change = (color: string) =>
                            setDraft({
                              ...draft,
                              ...(group
                                ? {
                                    group_colors: {
                                      ...draft.group_colors,
                                      [key]: color,
                                    },
                                  }
                                : { [key]: color }),
                            });
                          return (
                            <div className="native-color-row" key={key}>
                              <label>
                                {key === "icon_color"
                                  ? "Global icons"
                                  : key === "text_color"
                                    ? "Global text"
                                    : key.toUpperCase()}
                                <select
                                  aria-label={`Color ${key}`}
                                  value={value}
                                  onChange={(e) => change(e.target.value)}
                                >
                                  {group && (
                                    <option value="inherit">
                                      Inherit globals
                                    </option>
                                  )}
                                  {catalog.palette.map((p) => (
                                    <option key={p.value} value={p.value}>
                                      {p.label}
                                    </option>
                                  ))}
                                  {!catalog.palette.some(
                                    (p) => p.value === value,
                                  ) &&
                                    value !== "inherit" && (
                                      <option value={value}>{value}</option>
                                    )}
                                </select>
                              </label>
                              <input
                                aria-label={`Custom hex for ${key}`}
                                type="color"
                                value={cssColor(
                                  value,
                                  theme.groups[
                                    key as keyof typeof theme.groups
                                  ] || theme.text,
                                )}
                                onChange={(e) => change(e.target.value)}
                              />
                            </div>
                          );
                        },
                      )}
                      <div className="native-button-row">
                        <button
                          onClick={() => {
                            update({
                              icon_color: draft.icon_color,
                              text_color: draft.text_color,
                              group_colors: { ...draft.group_colors },
                            });
                            setDraft(null);
                            setStatus("✓ Colors applied to browser setup");
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
                  <h3>Local setup</h3>
                  <div className="native-button-row">
                    <button onClick={restore}>Restore browser save</button>
                    <button
                      onClick={() => {
                        setSettings(structuredClone(defaults));
                        setDraft(null);
                      }}
                    >
                      Reset to native defaults
                    </button>
                    <button
                      onClick={() => {
                        setSettings(structuredClone(saved));
                        setDraft(null);
                      }}
                    >
                      Cancel changes
                    </button>
                  </div>
                  {(logo.kind === "image" || logo.kind === "custom") && (
                    <div className="native-asset-fields">
                      <label>
                        Image path on your machine
                        <input
                          aria-label="Image path on your machine"
                          value={settings.image_path}
                          placeholder="/absolute/path/to/logo.png"
                          onChange={(e) =>
                            update({ image_path: e.target.value })
                          }
                        />
                      </label>
                      <label>
                        Image protocol
                        <select
                          aria-label="Image protocol"
                          value={settings.image_protocol}
                          onChange={(e) =>
                            update({ image_protocol: e.target.value })
                          }
                        >
                          {["chafa", "kitty-direct", "iterm", "sixel"].map(
                            (protocol) => (
                              <option key={protocol}>{protocol}</option>
                            ),
                          )}
                        </select>
                      </label>
                      {logo.kind === "custom" && (
                        <label>
                          Custom ASCII
                          <textarea
                            value={settings.custom_art}
                            onChange={(e) =>
                              update({ custom_art: e.target.value })
                            }
                          />
                        </label>
                      )}
                    </div>
                  )}
                </Panel>
              )}
            </section>
            <aside className="native-preview-column" aria-label="Live Preview">
              <Panel
                title={
                  <>
                    <span className="native-ok">●</span> Live Preview{" "}
                    <span className="native-dim">· {preset.name}</span>
                    <button
                      className="preview-expand"
                      aria-label={
                        previewExpanded
                          ? "Restore studio layout"
                          : "Expand preview"
                      }
                      onClick={() => setPreviewExpanded(!previewExpanded)}
                    >
                      ⛶ f
                    </button>
                  </>
                }
                className="native-live-panel"
              >
                {output === "preview" ? (
                  <NativePreview settings={active} />
                ) : (
                  <pre
                    className="native-json-output"
                    tabIndex={0}
                    aria-label="Generated Fastfetch JSONC"
                  >
                    {jsonc(active)}
                  </pre>
                )}
              </Panel>
              <Panel title="󰒓 Preview Options" className="native-options">
                <label>
                  <span>󰏘 Theme</span>
                  <select
                    aria-label="Theme"
                    value={settings.theme}
                    onChange={(e) => update({ theme: e.target.value })}
                  >
                    {catalog.themes.map((t) => (
                      <option key={t.id} value={t.id}>
                        {t.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>󰉉 Preset</span>
                  <select
                    aria-label="Preset"
                    value={settings.preset}
                    onChange={(e) => update({ preset: e.target.value })}
                  >
                    {presets.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>󰋔 Logo</span>
                  <select
                    aria-label="Logo"
                    value={settings.logo}
                    onChange={(e) => update({ logo: e.target.value })}
                  >
                    {catalog.logos.map((l) => (
                      <option key={l.id} value={l.id}>
                        {l.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>󰅩 Output</span>
                  <select
                    aria-label="Output"
                    value={output}
                    onChange={(e) => setOutput(e.target.value)}
                  >
                    <option value="preview">Terminal Preview</option>
                    <option value="jsonc">Generated JSONC</option>
                  </select>
                </label>
                <button
                  className="native-apply"
                  disabled={!!draft}
                  onClick={apply}
                >
                  ▶ Apply setup locally <kbd>A</kbd>
                </button>
              </Panel>
            </aside>
          </div>
          <footer className="native-footer">
            {[
              ["?", "Navigate"],
              ["↕", "Move"],
              ["Enter", "Select"],
              ["Tab", "Next Section"],
              ...footerKeys[screen],
            ].map(([k, label]) => (
              <span key={k + label}>
                <kbd>{k}</kbd> {label}
              </span>
            ))}
            <span className={dirty ? "native-warn" : "native-ok"}>
              {dirty ? "● Unsaved changes" : "✓ Applied setup"}
            </span>
            <span className="native-status" role="status">
              {status}
            </span>
            <button onClick={() => setHelp(!help)} aria-label="Keyboard help">
              ?
            </button>
          </footer>
          {presetDialog && (
            <form
              className="native-help preset-dialog"
              role="dialog"
              aria-label={
                presetDialog === "rename" ? "Rename preset" : "Delete preset"
              }
              onSubmit={(e) => {
                e.preventDefault();
                savePresetDialog();
              }}
            >
              <h3>
                {presetDialog === "rename" ? "Rename preset" : "Delete preset?"}
              </h3>
              <p>
                {presetDialog === "rename"
                  ? "Give this layout a name."
                  : `Type yes to delete “${preset.name}” from this browser setup.`}
              </p>
              <input
                aria-label={
                  presetDialog === "rename"
                    ? "Preset name"
                    : "Type yes to delete preset"
                }
                autoFocus
                maxLength={100}
                value={presetInput}
                onChange={(e) => setPresetInput(e.target.value)}
              />
              <div className="native-button-row">
                <button
                  type="submit"
                  disabled={
                    presetDialog === "rename"
                      ? !presetInput.trim()
                      : presetInput.toLowerCase() !== "yes"
                  }
                >
                  {presetDialog === "rename"
                    ? "Save preset name"
                    : "Confirm delete preset"}
                </button>
                <button type="button" onClick={() => setPresetDialog(null)}>
                  Cancel
                </button>
              </div>
            </form>
          )}
          {help && (
            <div
              className="native-help"
              role="dialog"
              aria-label="Keyboard shortcuts"
            >
              <h3>Keyboard shortcuts</h3>
              <p>
                1–6 sections · a apply locally · d copy preset (Presets) · s
                logo size · o output · f preview focus · Esc return · demo mode
                lives in Settings
              </p>
              <p>
                Tab and Shift+Tab move through controls. Text fields keep normal
                typing.
              </p>
              <button onClick={() => setHelp(false)}>Close help</button>
            </div>
          )}
        </div>
      </div>
      <div className="demo-note">
        <p>
          Interactive browser recreation · sample specs only · Apply saves
          locally, never to your system.{" "}
          <a href="/docs#behavior">Browser & native differences ↗</a>
        </p>
        <div>
          <button disabled={!!draft} onClick={copy}>
            Copy JSONC
          </button>
          <button
            disabled={!!draft}
            onClick={() => {
              download(jsonc(settings), "config.jsonc");
              setStatus("✓ JSONC download requested");
            }}
          >
            Download JSONC ↓
          </button>
        </div>
      </div>
      <p className="mobile-pan-hint">
        ↔ Pan across the app, or expand the studio. The desktop layout stays
        intact.
      </p>
    </section>
  );
}
