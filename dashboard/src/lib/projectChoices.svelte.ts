// The reader's choices about projects: pinned ones come first wherever
// projects are listed, and hidden ones are left out of those lists. Their
// chats stay in search and in every chat list. The choices are kept by the
// dashboard, beside its index, so they follow the reader between browsers;
// see crates/mash-dash's project_choices.rs.
import { createQuery, queryOptions, useQueryClient } from "@tanstack/svelte-query";
import { stableRead } from "./queries";
import type { Answer } from "./routes";
import { get, post } from "./services.svelte";

type Choices = Answer<"/api/projects/choices">;

export const projectChoicesOptions = queryOptions({
  ...stableRead,
  queryKey: ["project-choices"],
  queryFn: ({ signal }) => get("/api/projects/choices", signal),
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

/** `projects` without the hidden ones. */
export function withoutHidden<T extends { readonly key: string }>(
  projects: readonly T[],
  hidden: readonly string[],
): T[] {
  return projects.filter((project) => !hidden.includes(project.key));
}

/** The sidebar's short list: every pinned project, then the most recent
 * others that are not hidden, while there is room for `room` in all. */
export function sidebarProjects<T extends { readonly key: string }>(
  recent: readonly T[],
  choices: { readonly pinned: readonly string[]; readonly hidden: readonly string[] },
  room: number,
): T[] {
  const ordered = pinnedFirst(withoutHidden(recent, choices.hidden), choices.pinned);
  const pinnedShown = ordered.filter((project) => choices.pinned.includes(project.key)).length;
  return ordered.slice(0, Math.max(room, pinnedShown));
}

/** The choices, and pinning or hiding a project. A project is pinned or hidden,
 * never both: hiding unpins it and pinning shows it again. The lists change on
 * screen at once and go back if the dashboard refuses the change. */
export function createProjectChoices() {
  const client = useQueryClient();
  const choices = createQuery(() => projectChoicesOptions);
  const pinned = (): readonly string[] => choices.data?.pinned ?? [];
  const hidden = (): readonly string[] => choices.data?.hidden ?? [];
  async function save(next: { pinned: string[]; hidden: string[] }): Promise<void> {
    const before = choices.data;
    client.setQueryData<Choices>(["project-choices"], { ...next, editable: true });
    try {
      client.setQueryData<Choices>(["project-choices"], await post("/api/projects/choices", next));
    } catch {
      client.setQueryData<Choices | undefined>(["project-choices"], before);
    }
  }
  const without = (list: readonly string[], project: string) =>
    list.filter((key) => key !== project);
  return {
    get pinned(): readonly string[] {
      return pinned();
    },
    get hidden(): readonly string[] {
      return hidden();
    },
    /** Whether this dashboard accepts a change to the choices. */
    get editable(): boolean {
      return choices.data?.editable ?? false;
    },
    isPinned: (project: string): boolean => pinned().includes(project),
    isHidden: (project: string): boolean => hidden().includes(project),
    togglePin(project: string): Promise<void> {
      return save(
        pinned().includes(project)
          ? { pinned: without(pinned(), project), hidden: [...hidden()] }
          : { pinned: [...pinned(), project], hidden: without(hidden(), project) },
      );
    },
    toggleHidden(project: string): Promise<void> {
      return save(
        hidden().includes(project)
          ? { pinned: [...pinned()], hidden: without(hidden(), project) }
          : { pinned: without(pinned(), project), hidden: [...hidden(), project] },
      );
    },
  };
}
