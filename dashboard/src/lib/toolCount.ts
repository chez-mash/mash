// How many tools agents can call on Mash, for the count beside Tools in the
// rail. It is the same query the tool explorer reads (same key, same answer),
// so the listing is fetched once and both stay in step.
import { read } from "./api";
import { parseConsole } from "./console";
import { stableRead } from "./queries";

export const toolCountOptions = {
  ...stableRead,
  queryKey: ["console-tools"] as const,
  queryFn: async ({ signal }: { signal: AbortSignal }) =>
    parseConsole(await read("/api/console/tools", new URLSearchParams(), signal)),
  select: (listing: { readonly tools: readonly unknown[] }) => listing.tools.length,
};
