// The projects the reader pinned. They are kept by the dashboard, beside its
// index, so they follow the reader between browsers; see crates/mash-dash's
// pins.rs. Pinned projects come first wherever projects are listed.
import { createQuery, queryOptions, useQueryClient } from "@tanstack/svelte-query";
import { stableRead } from "./queries";
import type { Answer } from "./routes";
import { get, post } from "./services.svelte";

type Pins = Answer<"/api/pins">;

export const pinsOptions = queryOptions({
  ...stableRead,
  queryKey: ["pins"],
  queryFn: ({ signal }) => get("/api/pins", signal),
});

/** `projects` with the pinned ones first, in the order they were pinned,
 * then the rest in their own order. A pin with no chats in view is left out. */
export function pinnedFirst<T extends { readonly key: string }>(
  projects: readonly T[],
  pinned: readonly string[],
): T[] {
  const byKey = new Map(projects.map((project) => [project.key, project]));
  const first = pinned.flatMap((key) => byKey.get(key) ?? []);
  const rest = projects.filter((project) => !pinned.includes(project.key));
  return [...first, ...rest];
}

/** The sidebar's short list: every pinned project, then the most recent
 * others while there is room for `room` in all. */
export function sidebarProjects<T extends { readonly key: string }>(
  recent: readonly T[],
  pinned: readonly string[],
  room: number,
): T[] {
  const ordered = pinnedFirst(recent, pinned);
  const pinnedShown = ordered.filter((project) => pinned.includes(project.key)).length;
  return ordered.slice(0, Math.max(room, pinnedShown));
}

/** The pins, and pinning or unpinning a project. The list changes on screen at
 * once and goes back if the dashboard refuses the change. */
export function createPins() {
  const client = useQueryClient();
  const pins = createQuery(() => pinsOptions);
  const list = (): readonly string[] => pins.data?.projects ?? [];
  return {
    get projects(): readonly string[] {
      return list();
    },
    /** Whether this dashboard accepts a change to the pins. */
    get editable(): boolean {
      return pins.data?.editable ?? false;
    },
    has(project: string): boolean {
      return list().includes(project);
    },
    async toggle(project: string): Promise<void> {
      const before = pins.data;
      const projects = list().includes(project)
        ? list().filter((key) => key !== project)
        : [...list(), project];
      client.setQueryData<Pins>(["pins"], { projects, editable: true });
      try {
        client.setQueryData<Pins>(["pins"], await post("/api/pins", { projects }));
      } catch {
        client.setQueryData<Pins | undefined>(["pins"], before);
      }
    },
  };
}
