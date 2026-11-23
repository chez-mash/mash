<script lang="ts">
import Bookmark from "@lucide/svelte/icons/bookmark";
import Bot from "@lucide/svelte/icons/bot";
import ChevronRight from "@lucide/svelte/icons/chevron-right";
import Clock from "@lucide/svelte/icons/clock";
import Download from "@lucide/svelte/icons/download";
import Folder from "@lucide/svelte/icons/folder";
import History from "@lucide/svelte/icons/history";
import House from "@lucide/svelte/icons/house";
import Laptop from "@lucide/svelte/icons/laptop";
import MessagesSquare from "@lucide/svelte/icons/messages-square";
import Server from "@lucide/svelte/icons/server";
import Settings from "@lucide/svelte/icons/settings";
import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
import Wrench from "@lucide/svelte/icons/wrench";
import X from "@lucide/svelte/icons/x";
import { createQuery } from "@tanstack/svelte-query";
import { agents as agentInfo } from "../lib/agents";
import { asset } from "../lib/asset";
import { groupChats, projectLabel } from "../lib/browse";
import { catalogOptions, stableRead } from "../lib/queries";
import { kindIcons } from "../lib/recordIcons";
import { getRecord, label as kindLabel, kinds } from "../lib/records";
import { agentHref, machineHref, projectHref, recordsHref, router } from "../lib/route.svelte";
import { fleetCatalogOptions, serviceAccessOptions } from "../lib/services.svelte";
import AgentMark from "./AgentMark.svelte";
import CheckUpdates from "./CheckUpdates.svelte";

let { open, close }: { open: boolean; close: () => void } = $props();
const catalogue = createQuery(() => catalogOptions);
const chats = $derived(catalogue.data?.chats ?? []);
const projects = $derived(
  groupChats(
    chats.filter((chat) => chat.project),
    (chat) => chat.project,
  )
    .sort((a, b) => (b.latestMs ?? 0) - (a.latestMs ?? 0))
    .slice(0, 6),
);
const providers = $derived(groupChats(chats, (chat) => chat.agent));
const machines = $derived(groupChats(chats, (chat) => chat.machine));

const sections = [
  { id: "overview", label: "overview", href: "#/", icon: House, tile: "var(--tile-overview)" },
  {
    id: "chats",
    label: "chats",
    href: "#/search",
    icon: MessagesSquare,
    tile: "var(--tile-chats)",
  },
  { id: "agents", label: "agents", href: "#/agents", icon: Bot, tile: "var(--tile-agents)" },
  {
    id: "machines",
    label: "machines",
    href: "#/machines",
    icon: Laptop,
    tile: "var(--tile-machines)",
  },
  {
    id: "projects",
    label: "projects",
    href: "#/projects",
    icon: Folder,
    tile: "var(--tile-projects)",
  },
  { id: "memory", label: "memory", href: "#/records?", icon: Bookmark, tile: "var(--tile-memory)" },
  {
    id: "services",
    label: "services",
    href: "#/services",
    icon: Server,
    tile: "var(--tile-services)",
  },
  {
    id: "fleet",
    label: "fleet manager",
    href: "#/fleet",
    icon: SlidersHorizontal,
    tile: "var(--tile-fleet)",
  },
  {
    id: "console",
    label: "tool explorer",
    href: "#/console",
    icon: Wrench,
    tile: "var(--tile-tools)",
  },
  {
    id: "settings",
    label: "settings",
    href: "#/settings",
    icon: Settings,
    tile: "var(--tile-settings)",
  },
] as const;
type Section = (typeof sections)[number]["id"];
const expandable: readonly Section[] = ["chats", "agents", "machines", "projects", "memory"];

const route = $derived(router.current);
// The fleet manager is offered only where fleet control is set up. Its page
// still explains the setup to anyone who opens it directly.
const access = createQuery(() => serviceAccessOptions);
const fleet = createQuery(() => fleetCatalogOptions(access.data !== undefined));
const offered = $derived(
  sections.filter(
    (section) => section.id !== "fleet" || fleet.data?.enabled === true || route.page === "fleet",
  ),
);
const area = $derived.by((): Section | "" => {
  switch (route.page) {
    case "overview":
      return "overview";
    case "search":
    case "history":
    case "export":
    case "chat":
      return "chats";
    case "agents":
    case "agent":
      return "agents";
    case "machines":
    case "machine":
      return "machines";
    case "projects":
    case "project":
      return "projects";
    case "records":
    case "record":
      return "memory";
    case "services":
      return "services";
    case "fleet":
      return "fleet";
    case "console":
      return "console";
    case "settings":
    case "setup":
      return "settings";
    default:
      return "";
  }
});
// A memory's own page lights its kind once the memory is read; the query is
// the page's own, so this reads the same answer rather than asking again.
const opened = createQuery(() => {
  const id = route.page === "record" ? route.id : "";
  const node = route.page === "record" ? route.node : "";
  return {
    ...stableRead,
    queryKey: ["record", id, node, null, 0],
    enabled: route.page === "record",
    queryFn: ({ signal }: { signal: AbortSignal }) => getRecord(id, node, null, 0, signal),
  };
});
const openedKind = $derived.by(() => {
  const record = opened.data;
  const current = record?.revisions.find(
    (revision) => revision.revision === record.currentRevision,
  );
  return current?.details.kind ?? "";
});
// A row is the current page only while none of its children is.
const child = $derived.by((): string => {
  // Search results are not the recent chats: with a query, the chats row is
  // the current one.
  if (route.page === "search") return new URLSearchParams(route.params).get("q") ? "" : "recent";
  if (route.page === "history") return "history";
  if (route.page === "export") return "export";
  if (route.page === "agent") return route.agent;
  if (route.page === "machine") return route.id;
  if (route.page === "project") return route.name;
  if (route.page === "records") return new URLSearchParams(route.params).get("kind") ?? "";
  if (route.page === "record") return openedKind;
  return "";
});
let expanded = $state<readonly Section[]>([]);
// Arriving in a section opens it and folds the others, so the rail keeps to
// the section the reader is in and its last rows stay in view. While they
// stay in a section, what they open or close is theirs.
let arrived = "";
$effect(() => {
  if (area === arrived) return;
  arrived = area;
  expanded = area && expandable.includes(area) ? [area] : [];
});
// Where the rows are taller than the rail, the list fades at the edge that
// has more to scroll to, so rows out of sight are not mistaken for none.
let nav = $state<HTMLElement>();
let fade = $state({ above: false, below: false });
function measure(): void {
  if (!nav) return;
  const above = nav.scrollTop > 1;
  const below = nav.scrollHeight - nav.scrollTop - nav.clientHeight > 1;
  if (above !== fade.above || below !== fade.below) fade = { above, below };
}
$effect(() => {
  const list = nav;
  if (!list) return;
  measure();
  const resized = new ResizeObserver(measure);
  resized.observe(list);
  const changed = new MutationObserver(measure);
  changed.observe(list, { childList: true, subtree: true });
  return () => {
    resized.disconnect();
    changed.disconnect();
  };
});

// Something above the dashboard, such as the demo's banner, pushes the rail
// down until the page scrolls past it. The rail is shorter by that much
// meanwhile, so its foot stays on screen.
let aside = $state<HTMLElement>();
let pushed = $state(0);
$effect(() => {
  const rail = aside;
  if (!rail) return;
  const update = (): void => {
    const top = Math.max(0, Math.round(rail.getBoundingClientRect().top));
    if (top !== pushed) pushed = top;
  };
  update();
  window.addEventListener("scroll", update, { passive: true });
  window.addEventListener("resize", update);
  return () => {
    window.removeEventListener("scroll", update);
    window.removeEventListener("resize", update);
  };
});

// The reader can drag the edge to set the width, which this browser keeps.
const widths = { least: 180, usual: 252, most: 440 } as const;
function stored(): number {
  try {
    const width = Number(localStorage.getItem("mash.rail.width"));
    return width >= widths.least && width <= widths.most ? width : widths.usual;
  } catch {
    return widths.usual;
  }
}
let width = $state(stored());
let dragging = $state(false);
function resize(next: number): void {
  width = Math.round(Math.min(widths.most, Math.max(widths.least, next)));
  try {
    localStorage.setItem("mash.rail.width", String(width));
  } catch {
    // The width still applies for this visit.
  }
}
function grip(event: PointerEvent): void {
  if (event.button !== 0) return;
  event.preventDefault();
  const handle = event.currentTarget as HTMLElement;
  const origin = event.clientX - width;
  handle.setPointerCapture(event.pointerId);
  dragging = true;
  const move = (moved: PointerEvent) => resize(moved.clientX - origin);
  const done = () => {
    dragging = false;
    handle.removeEventListener("pointermove", move);
    handle.removeEventListener("pointerup", done);
    handle.removeEventListener("pointercancel", done);
  };
  handle.addEventListener("pointermove", move);
  handle.addEventListener("pointerup", done);
  handle.addEventListener("pointercancel", done);
}
function nudge(event: KeyboardEvent): void {
  if (event.key === "ArrowLeft") resize(width - 16);
  else if (event.key === "ArrowRight") resize(width + 16);
  else if (event.key === "Home") resize(widths.usual);
  else return;
  event.preventDefault();
}
function toggle(id: Section): void {
  expanded = expanded.includes(id) ? expanded.filter((item) => item !== id) : [...expanded, id];
}
// The chevron beside a row is for pointers; keyboards reach its pages from
// the row's own link, which opens and closes them with the arrow keys, so
// each row is one stop on the way through the rail.
function arrows(event: KeyboardEvent, id: Section): void {
  if (!expandable.includes(id)) return;
  const isOpen = expanded.includes(id);
  if (event.key === "ArrowRight" && !isOpen) toggle(id);
  else if (event.key === "ArrowLeft" && isOpen) toggle(id);
  else return;
  event.preventDefault();
}
</script>

<aside id="rail" class:open style:--rail-width="{width}px" bind:this={aside}>
  <div class="rail-in" style:--rail-pushed="{pushed}px">
  <a class="brand" href="#/" onclick={close}><img src={asset("mash-mark.svg")} alt="" /><span>mash<i aria-hidden="true">_</i></span></a>
  <button id="xdrawer" type="button" aria-label="Close navigation" onclick={close}><X size={18} aria-hidden="true" /></button>
  <nav class="nav" class:fade-above={fade.above} class:fade-below={fade.below} aria-label="Main" bind:this={nav} onscroll={measure}>
    {#each offered as section (section.id)}
      {@const on = section.id === area && !child}
      {@const isOpen = expanded.includes(section.id)}
      <div class="nav-row" class:on>
        <a class="nav-link" href={section.href} aria-current={on ? "page" : undefined} aria-keyshortcuts={expandable.includes(section.id) ? "ArrowRight ArrowLeft" : undefined} onclick={close} onkeydown={(event) => arrows(event, section.id)}><span class="nav-ico" style:background={section.tile}><section.icon size={14} aria-hidden="true" /></span>{section.label}</a>
        {#if expandable.includes(section.id)}
          <button class="chev" type="button" tabindex="-1" aria-label="{isOpen ? 'Collapse' : 'Expand'} {section.label}" aria-expanded={isOpen} onclick={() => toggle(section.id)}><ChevronRight size={14} /></button>
        {/if}
      </div>
      {#if isOpen}
        <div class="subs">
        {#if section.id === "chats"}
          <a class="sub" class:on={child === "recent"} href="#/search" aria-current={child === "recent" ? "page" : undefined} onclick={close}><Clock size={14} aria-hidden="true" /><span>recent chats</span></a>
          <a class="sub" class:on={child === "history"} href="#/history" aria-current={child === "history" ? "page" : undefined} onclick={close}><History size={14} aria-hidden="true" /><span>viewing history</span></a>
          <a class="sub" class:on={child === "export"} href="#/export" aria-current={child === "export" ? "page" : undefined} onclick={close}><Download size={14} aria-hidden="true" /><span>export chats</span></a>
        {:else if section.id === "agents"}
          {#each providers as provider (provider.key)}
            {@const agent = provider.chats[0]?.agent}
            {#if agent}<a class="sub" class:on={child === agent} href={agentHref(agent)} onclick={close}><AgentMark {agent} size="sm" /><span>{agentInfo[agent].name}</span></a>{/if}
          {/each}
        {:else if section.id === "machines"}
          {#each machines as machine (machine.key)}
            <a class="sub" class:on={child === machine.key} href={machineHref(machine.key)} onclick={close}><i class="dot"></i><span>{machine.key}</span></a>
          {/each}
        {:else if section.id === "projects"}
          {#each projects as project (project.key)}
            <a class="sub" class:on={child === project.key} href={projectHref(project.key)} title={project.key} onclick={close}><Folder size={14} aria-hidden="true" /><span>{projectLabel(project.key)}</span></a>
          {/each}
        {:else if section.id === "memory"}
          {#each kinds as kind (kind)}
            {@const KindIcon = kindIcons[kind]}
            <a class="sub" class:on={child === kind} href={recordsHref(new URLSearchParams({ kind }))} onclick={close}><KindIcon size={14} aria-hidden="true" /><span>{kindLabel(kind).toLowerCase()}s</span></a>
          {/each}
        {/if}
        </div>
      {/if}
    {/each}
    <div class="status" style:margin-top="auto" style:padding-top="14px">
      {#if catalogue.data}{catalogue.data.localNode} / {chats.length.toLocaleString()} chats{:else if catalogue.error}history unavailable{:else}loading…{/if}
      <span style:display="flex" style:margin-top="6px"><CheckUpdates checking={catalogue.isFetching} check={() => catalogue.refetch()} /></span>
    </div>
  </nav>
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div id="rail-grip" class:on={dragging} role="separator" aria-orientation="vertical" aria-label="Resize navigation" aria-valuemin={widths.least} aria-valuemax={widths.most} aria-valuenow={width} tabindex="0" onpointerdown={grip} onkeydown={nudge} ondblclick={() => resize(widths.usual)}></div>
</aside>
