<script lang="ts">
// Around every story: a query cache of its own and the transport that answers
// its components from made-up history, so a component that reads its own data
// is drawn as it is in the dashboard without a `mash dash` behind it.
import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
import type { Snippet } from "svelte";
import { stableRead } from "../lib/queries";
import { install } from "../lib/transport";
import { answering, type StoryData } from "./answers";

let { children, data }: { children: Snippet; data?: StoryData | undefined } = $props();

const client = new QueryClient({ defaultOptions: { queries: stableRead } });
// Before the story's components are made, so their first read is answered.
// svelte-ignore state_referenced_locally
{
  install(answering(data?.answers));
  const route = data?.route ?? "#/";
  if (location.hash !== route) {
    history.replaceState(history.state, "", route);
    dispatchEvent(new HashChangeEvent("hashchange"));
  }
  if (data?.surface) document.documentElement.dataset.surface = data.surface;
  else delete document.documentElement.dataset.surface;
  data?.seed?.(client);
  data?.before?.();
}
</script>

<QueryClientProvider {client}>{@render children()}</QueryClientProvider>
