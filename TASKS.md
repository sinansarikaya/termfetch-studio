# Project Tasks

<!-- AI-TEAM-TASKS-START -->
## AI Team Tasks

### Task: ui-final-gate - Final independent read-only gate with workspace frozen. Read tests/ui-evidence/source-review/corrections-review.json: prior reviewer approved both P2 fixes and reported no issues, but runner detected concurrent leader test-report write (now restored). Review actual corrections via source chunks install-lines-1-800.txt and install-lines-4001-4800.txt, tests/ui_regression.py and current tests/docker_matrix.py runtime_logo_rewrite block. Confirm updated matrix check captures runtime config before cleanup, still calls real fastfetch and checks fallback/type/width/height/cleanup (not weakened). Read final-regression.log, final-docker-smoke.json, final-startup.json and final-shellcheck.log. No code changes. PNG leader scope; screenshots SHA357cff predates only non-layout SGR/mktemp fixes, final source SHA356b78dc. Run compileall verification.
- **Status**: Done (Accepted)
- **Role**: code-reviewer
- **Provider**: codex
- **Leader Summary**: Independent source review clear; 10 regression tests, 4 Docker scenarios/112 checks, real Kitty evidence and scope inspected.
- **Changed Files**: []
- **Evidence Paths**: ["tests/ui-evidence/source-review/corrections-review.json", "tests/ui-evidence/source-review/install-lines-1-800.txt", "tests/ui-evidence/source-review/install-lines-4001-4800.txt", "tests/ui-evidence/source-review/install-lines-4801-5600.txt", "tests/ui_regression.py", "tests/docker_matrix.py", "tests/ui-evidence/final-regression.log", "tests/ui-evidence/final-docker-smoke.json", "tests/ui-evidence/final-startup.json", "tests/ui-evidence/final-shellcheck.log", "tests/ui-evidence/README.md"]
- **Accepted At**: 2026-10-21T22:24:18Z

### Task: ui-polish-review - Read-only independent review of five requested Bash UI improvements. Read tests/ui-polish-evidence/changes.patch, source-680-1225.txt (exact lines covering wrapper, geometry, shared renderer/input), source-2720-2865.txt (logo menu), tests/ui_regression.py and evidence README/logs. No need to read oversized full install.sh; inspect patch and exact source slices. Verify separate icon/text anchors, compact title/one footer, fixed right/bottom/hidden preview, persisted small/medium/large dimensions default medium, no selected-index or stty/signal regressions. Pay attention new batched raw input and deferred cleanup; meaningful unchanged tests pass. Final evidence source32b2875a. PNG verification leader scope; do not claim direct screenshot viewing. No writes, no agents. Report significant actionable issues with exactlines or clear acceptance. Run permitted compileall.
- **Status**: Done (Accepted)
- **Role**: code-reviewer
- **Provider**: codex
- **Leader Summary**: Five visual changes validated in source, 13 PTY tests, 5 resize repetitions, 112 Docker checks and real Kitty screenshots including terminal reopen.
- **Changed Files**: []
- **Evidence Paths**: ["tests/ui-polish-evidence", "tests/ui_regression.py"]
- **Accepted At**: 2026-10-21T22:40:35Z

### Task: preview-size-review - Independent read-only review of narrow follow-up. Read tests/preview-fix-evidence/changes.patch and evidence README, tests/ui_regression.py and tests/vm_preview_capture.py. Changes enlarge configured image bounds from20/14/30 to20/36/52 with same aspect-fit helper and terminal bounds; complete menu frame is rendered to private mktemp file then cat to stdout after ready, removed by scoped teardown. Assess correctness/security of frame file lifecycle and signal behavior, meaningful slow-preview regression baseline failure/new14passes, changed Docker numeric expectations reflect deliberate increased size not weakening. Native sampled frames are limited observation not universal flicker proof; PNG leader scope. No source edits. Return significant bugs or clear acceptance; run compileall.
- **Status**: Done (Accepted)
- **Role**: code-reviewer
- **Provider**: codex
- **Leader Summary**: Reviewed source, old-code reproduction, 14 PTY tests, 112 Docker checks and sampled native Kitty frames; larger logo and prepared-frame replacement validated within documented limits.
- **Changed Files**: []
- **Evidence Paths**: ["tests/preview-fix-evidence", "tests/ui_regression.py", "tests/vm_preview_capture.py", "tests/docker_matrix.py"]
- **Accepted At**: 2026-10-21T22:48:14Z

### Task: persistent-ui-colors-review - Independent read-only review. Read tests/live-colors-evidence/changes.patch and README, tests/ui_regression.py and tests/vm_preview_capture.py. Key fixes: application MAINSCRIPT owns one alternate-screen session with EXIT cleanup, selectors do not restore underlying buffer between submenus; per-key ED2 removed except initial/resize (before graphics upload). Kitty chafa base64 protocol action T becomes upload-only t with owned random32bit ID; upload outside sync frame, then place new and delete old ID inside short frame. Assess payload/control/header safety, tempdir lifecycle, scoped deletes, fallback, signals and root exit. Color editor is one page draft: arrows change only in-memory fields, inline sample, Save commits, Q cancels, reset draft, Theme/Default normalized per field in six preset generators. Check actual-app transition test, 21 tests, real-fastfetch all-color configs4cases116checks, sampled graphics AND menu-text ROIs. PNGs leader scope; native data is sampled not universal guarantee. No writes. Report significant actionable issues or clear acceptance; run permitted compileall.
- **Status**: Done (Accepted)
- **Role**: code-reviewer
- **Provider**: codex
- **Leader Summary**: Source, 21 PTY tests including real app session/signal behavior, 116 Docker checks, native samples and live-color PNGs inspected; no significant open findings.
- **Changed Files**: []
- **Evidence Paths**: ["tests/live-colors-evidence", "tests/ui_regression.py", "tests/vm_preview_capture.py", "tests/docker_matrix.py"]
- **Accepted At**: 2026-10-22T08:37:46Z

### Task: group-colors-startup-final-review - Independent read-only review. Read tests/group-colors-evidence/changes.patch and README.md/report, tests/shell_integration.py and tests/ui_regression.py. Five semantic group colors override both key/output in all six preset generators; Inherit/Theme/Default and 16 named RGB palette. Advanced duplicate removed with reindexed dispatch. Shell integration preserves personal code, detects direct external startup call and suppresses own call, guards repeated sourcing/nested managed shells using TTY marker; noninteractive quiet. Assess real bugs/data preservation, invalid values, correct live sample and save/cancel/reset, preset mapping and reports. 25 PTY tests, four Docker scenarios/120 checks with actual fastfetch, old-source dedup failure, native Arch/Hyprland/Kitty screenshots inspected by leader. Do not claim direct PNG review. Full install.sh may exceed read cap; patch and source snippets available. No writes, no agents. Run allowed compileall. Return significant actionable findings or clear acceptance. Prior initial-review.json found a real P2: Inherit sample did not match selected preset. Now color_preview_base_colors generates the selected preset once in a private mktemp config copy, forces only that copy to theme and inherit, reads actual module key/output colors and caches first representative per semantic group for the session. The global BOTH theme + group Inherit case uses these cache values. No navigation regeneration or writes to actual config. Review cleanup/copy/dynamic CONFIG_FILE scope and performance/error behavior. New test compares Minimal/Time and Gaming/Hardware preview SGR to generated JSON; before-inherit.log fails, final regression25passes. Read source-editor.txt includes helper. Original startup and group override source unchanged since initial review. Source/JSON reports and native screenshot hashes match final installer. Full workspace is frozen; no writes.
- **Status**: Done (Accepted)
- **Role**: code-reviewer
- **Provider**: codex
- **Leader Summary**: P2 Inherit mismatch reproduced and corrected; independent review clear; 25 PTY tests, 120 Docker checks, shell deduplication and real native Kitty screenshots verified.
- **Changed Files**: []
- **Evidence Paths**: ["tests/group-colors-evidence/source-editor.txt", "tests/group-colors-evidence/source-startup.txt", "tests/group-colors-evidence/changes.patch", "tests/group-colors-evidence/README.md", "README.md"]
- **Accepted At**: 2026-10-22T09:13:28Z
<!-- AI-TEAM-TASKS-END -->

## Initial attempt (not accepted)
- terminal-ui-fix: timed out at 600s; incomplete patch failed Docker regressions and was not accepted.
- terminal-ui-review: original dependency failed; superseded by final review of leader integration.

## Notes
- Keep Bash for this scoped fix; assess larger TUI migration only if native terminal evidence shows the bounded approach is insufficient.
- Host daily-use installation/settings/fonts are outside this task. No release/version change.
- User screenshots are private input and will not be copied into Git.
- Web/Playwright/SEO/mobile gates N/A: native Bash TUI. Required gates: generated Bash syntax/ShellCheck, Docker PTY regressions, real Kitty VM rendering, independent review.

## Done
- Leader integration: single bounded Bash renderer, pixel-aware image sizing, scoped terminal cleanup and actual-wrapper tests.
- Docker UI regressions 10/10; Ubuntu/Arch smoke 4/4 (112 checks); Bash/Zsh/Fish startup and generated Bash/ShellCheck gates passed.
- Real Kitty VM: production app installed as normal user in temporary home; 80x24 and 40x12, image changes, resize preservation and aligned theme columns captured.

## Final acceptance
- ui-final-gate: accepted after independent review; no open significant findings. Leader inspected source, logs and real PNGs. No host installation changed.

## Notes (integration)
- Initial global EXIT trap polluted noninteractive output; removed and replaced with menu-scoped handlers restored on exit.
- Only-width fastfetch experiment timed out; explicit bounded dimensions retained with pixel metadata and integer-cell aspect fitting.
- New graphics proof is Arch/Hyprland/Kitty; host CachyOS/Niri application remains untouched.

## Review history
- ui-final-review and ui-source-completion: source read limit/protected path blocked; ordinary source chunks supplied.
- ui-source-access-review: two P2 findings, corrected and regression-tested (old code fails, new passes).
- ui-corrections-review: all source checks Pass, no open issues; runner rejected ownership because leader concurrently ran startup test that rewrote its report. Report restored; final gate runs with workspace frozen.
- Final source 356b78dc…: 10 PTY tests and 4 Docker scenarios /112 checks passed; final startup and ShellCheck passed. Native screenshots retain original 357cff… source provenance.

## UI polish — 22 October 2026
- All five requested visual improvements implemented in Bash; medium remains the default.
- Isolated Docker tests and native Kitty VM evidence in tests/ui-polish-evidence/.
- Host installation and settings remain untouched; source version stays 1.0.0.
- ui-polish-review accepted: no open significant findings; 13 PTY tests, 5 resize repeats, 4 Docker scenarios/112 checks, native screenshots and reopen persistence verified.

## Preview follow-up
- Reproduced slow-preview blanking; enlarged image sizes and moved full-frame preparation off screen.
- 14 regression tests, 112 Docker checks and native Kitty sampled frames passed. preview-size-review accepted; no significant open findings.

## Persistent UI and live colors
- Corrected the earlier incomplete flicker fix: remove per-key ED2 and preserve one screen across actual application submenu transitions.
- Kitty upload/place/owned-ID retirement, scoped symbols fallback, simple live color draft editor with Save/Cancel.
- 21 tests and 4 Docker scenarios/116 checks passed; native samples include menu text and graphics.
- persistent-ui-colors-review accepted; source/native/Docker hashes match. No significant open findings. No host application/settings changes.

## Group colors and startup — 22 October 2026
- Reproduced repeated startup output with the prior installer; host read-only evidence showed legacy external Fish startup plus a managed block. Personal blocks are preserved.
- Consolidated color editor, five semantic group overrides, local RGB palette; no external color service or new runtime dependency.
- 25 PTY tests, 4 Docker scenarios/120 checks, Bash/Zsh/Fish startup/deduplication and native Arch/Hyprland/Kitty screenshots passed.
- Scope: host application/settings untouched; version remains 1.0.0. Other graphical terminals need manual validation.
- Reviewer selection: code-reviewer used for independent Bash/data-preservation review; automatic surgeon-reviewer metadata suggestion was narrower than the required full regression/evidence gate.
- group-colors-startup-final-review accepted after a frozen-workspace review; no significant open findings.

- First review found P2 Inherit sample/preset mismatch; reproduced with a failing test and corrected using a private generated config cache. Initial review not accepted; corrected source passed the bounded final review.
