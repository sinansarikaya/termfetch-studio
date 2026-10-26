# syntax=docker/dockerfile:1
FROM node:24-alpine@sha256:06d7adf3e0ef61a13f1d28682726366f97a15aa1888c934d3ecec86c33918b53 AS termfetch-studio-builder
WORKDIR /termfetch-studio
COPY website/package.json website/package-lock.json ./website/
RUN --mount=type=cache,target=/root/.npm npm ci --prefix website
COPY Cargo.toml ./
COPY src/domain/ ./src/domain/
COPY src/adapters/fastfetch/config.rs ./src/adapters/fastfetch/config.rs
COPY assets/brand/ ./assets/brand/
COPY website/ ./website/
RUN npm run build --prefix website

FROM nginxinc/nginx-unprivileged:stable-alpine@sha256:f5116ed4606f3eb015dd4a8e28cb9b351476dc0ad4bc6e9f93b19cec130cdf03 AS termfetch-studio-web
COPY --chown=101:101 deploy/termfetch-studio-nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=termfetch-studio-builder --chown=101:101 /termfetch-studio/website/dist/ /usr/share/nginx/html/
USER 101:101
EXPOSE 8080
STOPSIGNAL SIGQUIT
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 CMD wget -q -O /dev/null http://127.0.0.1:8080/healthz || exit 1
ENTRYPOINT ["nginx", "-g", "daemon off;"]
