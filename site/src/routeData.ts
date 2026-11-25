import { defineRouteMiddleware } from "@astrojs/starlight/route-data";
import { tagline } from "./brand.ts";

type Route = App.Locals["starlightRoute"];

export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute;
  if (route.isFallback) keepOutOfSearch(route);
  if (route.entry.data.template === "splash" && route.entry.data.hero) brandLanding(route);
});

// A guide that is not translated yet still has a page in each language, which
// shows the English text. Search keeps one index per language, so such a page
// would put English results in, say, the Japanese search. Only the guide in
// its own language is indexed: the English text is found from English pages.
function keepOutOfSearch(route: Route): void {
  // The entry is shared with the English page, so it is copied, not changed.
  route.entry = { ...route.entry, data: { ...route.entry.data, pagefind: false } };
}

// The landing page's title and link previews carry the tagline: the title is
// "Mash: Supercharged context for your agents." and a shared link shows the
// tagline under it. The page's own description, the longer paragraph under
// the headline, stays the search engines' description.
function brandLanding(route: Route): void {
  const title = `Mash: ${tagline}`;
  for (const entry of route.head) {
    if (entry.tag === "title") entry.content = title;
    const key = entry.attrs?.property ?? entry.attrs?.name;
    if (key === "og:title" || key === "twitter:title")
      entry.attrs = { ...entry.attrs, content: title };
    if (key === "og:description" || key === "twitter:description")
      entry.attrs = { ...entry.attrs, content: tagline };
  }
}
