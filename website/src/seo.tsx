import { useEffect } from "react";
import { useLocation } from "react-router-dom";
import { getSeoMetadata, serializeJsonLd } from "./seo-data";

export function Seo() {
  const location = useLocation();
  useEffect(() => {
    const meta = getSeoMetadata(location.pathname);
    document.title = meta.title;
    const setMeta = (key: string, value: string, property = false) => {
      const attribute = property ? "property" : "name";
      let element = document.head.querySelector<HTMLMetaElement>(`meta[${attribute}="${key}"]`);
      if (!element) { element = document.createElement("meta"); element.setAttribute(attribute, key); document.head.appendChild(element); }
      element.content = value;
    };
    setMeta("description", meta.description);
    setMeta("robots", meta.noindex ? "noindex,follow" : "index,follow,max-image-preview:large");
    for (const [key, value] of Object.entries({ "og:title": meta.title, "og:description": meta.description, "og:url": meta.canonical, "og:image": meta.image })) setMeta(key, value, true);
    for (const [key, value] of Object.entries({ "twitter:title": meta.title, "twitter:description": meta.description, "twitter:image": meta.image })) setMeta(key, value);
    let canonical = document.head.querySelector<HTMLLinkElement>('link[rel="canonical"]');
    if (!canonical) { canonical = document.createElement("link"); canonical.rel = "canonical"; document.head.appendChild(canonical); }
    canonical.href = meta.canonical;
    let data = document.getElementById("termfetch-structured-data");
    if (!data) { data = document.createElement("script"); data.id = "termfetch-structured-data"; data.setAttribute("type", "application/ld+json"); document.head.appendChild(data); }
    data.textContent = serializeJsonLd(meta.structuredData);
  }, [location.pathname]);
  return null;
}

export default Seo;
