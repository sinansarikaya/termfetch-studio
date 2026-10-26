# TermFetch Studio website and shared brand

## Done

- Website created at website/ in this repository; original prototype preserved.
- Interactive browser studio uses Rust catalogs: themes, presets, logos, modules, semantic colors, local apply/restore and JSONC export.
- Downloads, Docs, Getting Started and Releases pages show verified public v2.0.0 downloads separately from source 2.1.0. Four package-manager channels remain pending.
- Original TF SVG brand with deterministic PNG/ICO/ICNS assets, website favicon, Linux/Windows launchers, Windows Terminal profile and Windows EXE resource embedding.
- AUR/RPM recipes remain compatible with existing v2.0.0 archives; future release archives require brand assets.
- Namespaced nonroot Dockerfile/Compose, Nginx health/cache/security/SPA configuration, website CI and deployment documentation.
- 111 Rust tests, 3 brand tests, 2 packaging tests with multiple fixtures, real published archive staging, Clippy and rustfmt passed.
- Website build/lint, 360 catalog combinations, zero npm advisories and eight headed browser groups passed. Independent native/container review passed.

## In progress

- Final website production image, source and evidence review.
- GitHub push, Dokploy MainServer deployment to termfetch-studio.sinansarikaya.dev and live checks.
- Separate Codex agent: Turkish horizontal/vertical OpenMontage promo in ~/youtube/videos/termfetch-studio-promo-20261026.

## Notes

- User authorized the same GitHub repo, Docker, existing GitHub/Dokploy connection and production domain. Service names must be prefixed termfetch-studio-.
- All workers use Codex. An initial CLI ownership failure came from the leader updating TASKS during the run. Cargo PATH and later capacity thresholds blocked CLI workers. Work continued through declared native Codex delegation without changing providers or thresholds.
- The video agent ignores prior TermFetch video folders. The user requires fake/demo system specs in all videos/images; actual app demo mode is authorized and must remain on. YouTube upload is not authorized.
- Windows/macOS native display is not proven by Linux/browser checks. Windows CI must confirm EXE linking. The macOS .command remains a Terminal document; ICNS is supplied. No app version bump, release or package-manager submission was requested.

### Task: website-release-acceptance - Follow-up independent read-only review: previous website-production-final-review confirmed website/demo/export/routes/metadata/privacy/browser/container sources and evidence Pass. Only root README pending package commands and unpublished archive names needed correction. Verify current README Installation explicitly distinguishes published2.0.0 from source2.1.0, removes pending TermFetch brew/yay commands, links real2.0archives and limits newer installer/startup claims to source. Additional verification: actual public Windows ZIP SHA checked,20entries and NOinstall.cmd (qa/website-release/windows-release.json). Website distribution.ts Windows note now correctly says manual extraction/runEXE/separateFastfetch. Root rebuilt localwebsite AND productionDocker after copy change: current dist index hash matches container.json; healthy101:101readonly. New actual headed test againstproductioncontainer passed all8groups. Evidence qa/website-release/{browser.json,container.json,docker-build.log,build.log,browser-production.log,windows-release.json}. Native/container independent review Pass qa/native-brand/independent-review.json; full111Rust+3assets+packaging tests already recorded. Scope calls to verifying those corrections and new artifact proof, then confirm all original criteria. Root will not mutate repo during review. Video task writes only external youtube workspace. Run git diff --check, no mutations/otheragents/secrets. Return clear Pass with no open issues if correct; report material issues otherwise. Native Windows/macOS rendering and not-yet-live publication are explicitly out of this acceptance (N/A). User already authorized production deployment, which follows this review.
- **Status**: Done (Accepted)
- **Role**: code-reviewer
- **Provider**: codex
- **Leader Summary**: Independent review Pass; actual production container, eight headed browser groups, build/lint/catalog and distribution corrections verified.
- **Changed Files**: []
- **Evidence Paths**: ["qa/website-release/browser.json", "qa/website-release/container.json", "qa/website-release/docker-build.log", "qa/website-release/build.log", "qa/website-release/browser-production.log", "qa/website-release/windows-release.json", "qa/native-brand/independent-review.json"]
- **Accepted At**: 2026-10-26T04:06:17Z
