/* eslint-disable react-refresh/only-export-components -- Build-only SSR entry; never loaded by the browser or Fast Refresh. */
import { renderToString } from "react-dom/server";
import { StaticRouter } from "react-router-dom";
import { Site } from "./App";
import { ROUTES, getSeoMetadata, renderSeoHead } from "./seo-data";

export { ROUTES, getSeoMetadata };

export function render(pathname: string) {
  return {
    html: renderToString(<StaticRouter location={pathname}><Site /></StaticRouter>),
    head: renderSeoHead(pathname),
    metadata: getSeoMetadata(pathname),
  };
}
