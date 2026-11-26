// What a story's components are told when they ask the dashboard something.
// Every request is answered in the browser by the demo's made-up history
// (src/demo); a story may answer some routes differently, to show a state the
// demo never reaches, such as a machine that stopped answering.
import type { QueryClient } from "@tanstack/svelte-query";
import { transport as demo } from "../demo/backend";
import type { Answer, ReadPath } from "../lib/routes";
import type { Transport } from "../lib/transport";

/** A story's own answers, by route. */
export type Answers = { readonly [P in ReadPath]?: Answer<P> };

/** What a story sets as `parameters.data`. */
export interface StoryData {
  readonly answers?: Answers;
  /** Where the dashboard's router is, such as "#/records". */
  readonly route?: string;
  /** "native" draws in the system's look, as the quick panel's window does. */
  readonly surface?: "native";
  /** Puts what a query would have read into the cache before the story draws. */
  readonly seed?: (client: QueryClient) => void;
  /** Runs before the story draws, for state kept outside the components. */
  readonly before?: () => void;
}

/** The demo's transport with a story's own answers to reads laid over it. */
export function answering(answers: Answers = {}): Transport {
  const own: Readonly<Record<string, unknown>> = answers;
  return (path, init) => {
    const { pathname } = new URL(path, "http://story.invalid");
    if ((init.method ?? "GET") !== "GET" || !Object.hasOwn(own, pathname)) return demo(path, init);
    return Promise.resolve(
      new Response(JSON.stringify(own[pathname]), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );
  };
}
