# Publish the website with Dokploy

The React site lives in [`website/`](../website/README.md) beside the Rust app.
The root [`Dockerfile`](../Dockerfile) builds it from the same Git revision as
the Rust catalog and shared brand. No database, persistent volume or application
secret is required.

## Build and check locally

Run from the repository root:

```sh
docker build -t termfetch-studio-web:local .
docker run --rm --name termfetch-studio-web-local \
  --read-only --tmpfs /tmp:rw,size=32m --cap-drop=ALL \
  --security-opt=no-new-privileges \
  -p 127.0.0.1:18080:8080 termfetch-studio-web:local
```

In another terminal:

```sh
curl --fail http://127.0.0.1:18080/healthz
curl --fail http://127.0.0.1:18080/docs
docker inspect --format '{{.Config.User}}' termfetch-studio-web-local
```

The user must be `101:101`. Check that missing asset URLs return 404 and that
`/downloads`, `/docs`, `/docs/getting-started` and `/releases` survive a direct
request and browser reload. Run the website's headed browser regression as
described in its README. Container smoke checks do not replace that browser test.

[`docker-compose.yml`](../docker-compose.yml) provides a single
`termfetch-studio-web` service with an application-specific image name,
read-only filesystem and temporary `/tmp`. It only exposes port 8080 to the
container network. The local command above deliberately binds a loopback port;
production uses Dokploy's existing reverse proxy instead.

## Configure Dokploy

Use the existing GitHub integration and deployment server. The Application
workflow builds the root Dockerfile; importing Compose is optional.

| Field | Value | Purpose |
| --- | --- | --- |
| Project | TermFetch Studio | Separate project for this product |
| Environment | production | Website deployment |
| Deployment server | MainServer | Existing server |
| Application name / app name | termfetch-studio-web | Namespaced Docker service |
| Source | Existing GitHub provider | Existing installation access |
| Owner / repository | sinansarikaya / termfetch-studio | Same repository as native app |
| Branch / trigger | main / push | Deploy reviewed Git updates |
| Build path | / | Monorepo context |
| Build type / Dockerfile | Dockerfile / Dockerfile | Root production build |
| Docker context | . | Includes Rust catalog and shared brand inputs |
| Container port | 8080 | Nonroot Nginx listener |
| Host / path | termfetch-studio.sinansarikaya.dev / / | Public site |
| HTTPS / certificate | Enabled / Let's Encrypt | Existing proxy terminates TLS |

Watch paths can be limited to `website/**`, `assets/brand/**`, `src/domain/**`,
`src/adapters/fastfetch/config.rs`, `Cargo.toml`, `Dockerfile`, `.dockerignore`
and `deploy/**`. The website build checks Rust source hashes and fails on
unreviewed catalog drift. Keep the shared catalog snapshot current when changing
those Rust files; instructions are in the website README.

Do not create another Traefik/proxy, shared network, database or Redis service.
Do not use generic service names such as `web`. Let Dokploy manage its existing
network and domain routing. Its [domain documentation](https://docs.dokploy.com/docs/core/domains)
describes the host, path, HTTPS and container-port fields.

## Verify a production deployment

Before publishing, verify build, lint, catalog consistency, browser interactions
and the production container. Push the reviewed source revision, configure the
application and domain, and deploy that revision through Dokploy.

```sh
curl --fail https://termfetch-studio.sinansarikaya.dev/healthz
curl --fail --head https://termfetch-studio.sinansarikaya.dev/docs
curl --fail https://termfetch-studio.sinansarikaya.dev/robots.txt
curl --fail https://termfetch-studio.sinansarikaya.dev/sitemap.xml
```

Check the certificate, live routes, shared icon assets, console errors and an
actual JSONC download. Confirm the deployed Git commit in Dokploy's deployment
record. Save that commit, image identity and smoke results with the release QA
record. A queued build is not a successful deployment.

The runtime has `/healthz`, a graceful SIGQUIT stop, HTML revalidation, immutable
hashed JavaScript/CSS caching, 24-hour brand/image caching and explicit missing
asset 404s. Security headers permit the inline styles required by the color
preview while scripts load only from the site itself. There is no runtime API.

## Roll back

Keep the previous successful Dokploy deployment and image until the new site
passes its live checks. Use Dokploy's rollback action when that deployment has
a retained image. Otherwise revert the website change in Git, review the revert
and deploy it from `main`; do not rewrite Git history. Recheck `/healthz`, routes
and the visible application after rollback. This static deployment has no
database migrations or volumes to reverse.

Base images are pinned by multi-architecture digest. Refresh those digests
deliberately, then repeat the build and runtime checks. Do not replace them with
an unpinned `latest` tag during deployment troubleshooting.
