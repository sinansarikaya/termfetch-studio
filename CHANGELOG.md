# Changelog

## Unreleased

- Avoid duplicate automatic fastfetch output across managed shell startup, repeated sourcing and nested shells; preserve external personal startup blocks and skip an additional managed fetch when a direct call exists.
- Consolidate color editing in the main Icon & Text Colors page, removing the duplicate Advanced Settings entry.
- Add draft/live colors for OS, Desktop, Hardware, Network and Time groups across all six presets; preserve layout and support Inherit/Theme/Default.
- Extend the local palette with 16 named RGB colors and verify group configuration with real fastfetch, PTY interactions and a native Wayland Kitty VM.

- Keep one alternate-screen session across interactive submenus so the underlying fastfetch page is not exposed between selections.
- Stop clearing the whole screen on arrow navigation; upload Kitty previews without displaying them, then swap and delete only owned image IDs.
- Replace color mode/pickers/separate preview with one live draft editor, save/cancel and per-field Theme/Default handling.
- Validate all preset color combinations with real fastfetch and add application-level transition/signal regressions.

- Enlarge medium/large image-logo limits while retaining source proportions and terminal bounds.
- Prepare complete menu frames before display so slow preview conversion does not blank the visible preview.

- Simplify menu headers and navigation hints; draw icons and labels in separate cell columns.
- Keep previews on the right in wide windows, below the list in narrower windows, and hidden in very small windows.
- Add persistent small/medium/large image-logo sizing with bounded aspect-preserving dimensions.
- Preserve custom preset modules when changing logo settings.
- Stabilize menu input across resize and defer signal cleanup until the input read finishes.

- Preserve image logo aspect using source and terminal cell measurements with bounded fallback sizes.
- Use a shared viewport renderer for menus, paginate within small terminals, and redraw on resize.
- Anchor theme columns to terminal cells and constrain native image previews to their allotted area.
- Restore terminal state and caller traps on menu exit; keep noninteractive library loading quiet.
- Generate valid truecolor SGR for preset color dots and isolate/clean runtime configs per invocation.
- Add isolated PTY regressions and real Kitty screenshot evidence for the UI fixes.

- Recognize distribution families through `ID` and `ID_LIKE`, including openSUSE Tumbleweed, while keeping version support tied to tested images.
- Pin and verify the upstream fastfetch Debian package for x86_64/aarch64, check download and package tools before use, and reject unsupported architectures.
- Report optional dependency failures accurately and support both ImageMagick 6 and 7 command names.
- Follow the current terminal's parent processes and session signals; unknown terminals fall back to text.
- Preserve a backup of legacy JSONC and remove unsupported `keyShow` fields on reinstall.
- Require fastfetch >= 2.42.0 for RGB configuration without replacing an older existing installation.
- Use explicit RGB theme colors for fastfetch and fix malformed theme color values.
- Generate valid ASCII fallback logo types and use Kitty's direct PNG protocol and Konsole's iTerm protocol with explicit dimensions to avoid a blocking terminal query.
- Install missing `awk` and check essential utilities; preserve the existing configuration when a settings write fails.
- Run Bash/Zsh/Fish theme and fastfetch startup only in interactive terminals, keeping SSH/SCP sessions quiet.
- Create backups without including older archives, handle filenames with spaces and invalid restore choices, and propagate backup/restore errors.
- Generate the initial fastfetch configuration during installation and preserve user configuration permissions on reinstall.
- Remove shadowed color-setting functions and the unused binary-download helper.
- Remove the `perl` requirement during uninstall and leave unrelated system and user files intact.
- Add isolated Docker, shell startup and terminal ancestry tests, VM controls and real graphical screenshot evidence.

The application version remains 1.0.0; the published v1.0.0 release is unchanged.
