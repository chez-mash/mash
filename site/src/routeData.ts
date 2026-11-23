import { defineRouteMiddleware } from "@astrojs/starlight/route-data";

// A guide that is not translated yet still has a page in each language, which
// shows the English text. Search keeps one index per language, so such a page
// would put English results in, say, the Japanese search. Only the guide in
// its own language is indexed: the English text is found from English pages.
export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute;
  if (!route.isFallback) return;
  // The entry is shared with the English page, so it is copied, not changed.
  route.entry = { ...route.entry, data: { ...route.entry.data, pagefind: false } };
});
