<script lang="ts">
// The navigation rail under the top bar: the main pages with their counts,
// the tools that manage Mash, the projects pinned for quick reach, and a
// status line at the foot. On a phone it is a sheet that drops from the top,
// with the viewing switcher above the pages.

import { createQuery } from "@tanstack/svelte-query";
import BrainIcon from "phosphor-svelte/lib/BrainIcon";
import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
import ChatIcon from "phosphor-svelte/lib/ChatIcon";
import ClockCounterClockwiseIcon from "phosphor-svelte/lib/ClockCounterClockwiseIcon";
import ClockIcon from "phosphor-svelte/lib/ClockIcon";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import GearSixIcon from "phosphor-svelte/lib/GearSixIcon";
import HardDrivesIcon from "phosphor-svelte/lib/HardDrivesIcon";
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import PlugIcon from "phosphor-svelte/lib/PlugIcon";
import PushPinIcon from "phosphor-svelte/lib/PushPinIcon";
import RobotIcon from "phosphor-svelte/lib/RobotIcon";
import SquaresFourIcon from "phosphor-svelte/lib/SquaresFourIcon";
import UploadSimpleIcon from "phosphor-svelte/lib/UploadSimpleIcon";
import WrenchIcon from "phosphor-svelte/lib/WrenchIcon";
import XIcon from "phosphor-svelte/lib/XIcon";
import Icon from "$lib/components/Icon.svelte";
import { asset } from "../lib/asset";
import { groupChats, projectLabel } from "../lib/browse";
import { askForSearch } from "../lib/findField";
import { createProjectChoices, sidebarProjects } from "../lib/projectChoices.svelte";
import { catalogOptions } from "../lib/queries";
import { projectHref, router } from "../lib/route.svelte";
import { createScoped } from "../lib/scoped.svelte";
import { fleetCatalogOptions, serviceAccessOptions } from "../lib/services.svelte";
import { toolCountOptions } from "../lib/toolCount";
import RailFoot from "./RailFoot.svelte";
import ViewingSwitcher from "./ViewingSwitcher.svelte";

let { open, close }: { open: boolean; close: () => void } = $props();
const catalogue = createQuery(() => catalogOptions);
// Counts follow the viewing scope, as the pages they lead to do. Machines
// lists every machine whatever the scope, so its count does too.
const scoped = createScoped();
const chats = $derived(catalogue.data?.chats ?? []);
const seen = $derived(scoped.data?.chats ?? []);
const choices = createProjectChoices();
// Every pinned project, then the most recent others not hidden, while there is room.
const projects = $derived(
  sidebarProjects(
    groupChats(
      chats.filter((chat) => chat.project),
      (chat) => chat.project,
    ).sort((a, b) => (b.latestMs ?? 0) - (a.latestMs ?? 0)),
    choices,
    6,
  ),
);
const allPinned = $derived(
  projects.length > 0 && projects.every((project) => choices.isPinned(project.key)),
);
const tools = createQuery(() => toolCountOptions);
const counts = $derived<Readonly<Record<string, number | undefined>>>({
  chats: scoped.data ? seen.length : undefined,
  machines: catalogue.data ? groupChats(chats, (chat) => chat.machine).length : undefined,
  agents: scoped.data ? groupChats(seen, (chat) => chat.agent).length : undefined,
  console: tools.data,
});

interface Row {
  readonly id: string;
  readonly label: string;
  readonly href: string;
  readonly icon: typeof BrainIcon;
}
const main: readonly Row[] = [
  { id: "overview", label: "Overview", href: "#/", icon: SquaresFourIcon },
  { id: "search", label: "Search", href: "#/search", icon: MagnifyingGlassIcon },
  { id: "chats", label: "Chats", href: "#/search", icon: ChatIcon },
  { id: "memory", label: "Memory", href: "#/records?", icon: BrainIcon },
  { id: "machines", label: "Machines", href: "#/machines", icon: LaptopIcon },
  { id: "agents", label: "Agents", href: "#/agents", icon: RobotIcon },
  { id: "services", label: "Services", href: "#/services", icon: HardDrivesIcon },
  { id: "settings", label: "Settings", href: "#/settings", icon: GearSixIcon },
];
const manage: readonly Row[] = [
  { id: "console", label: "Tools", href: "#/console", icon: PlugIcon },
  { id: "export", label: "Export", href: "#/export", icon: UploadSimpleIcon },
  { id: "fleet", label: "Fleet manager", href: "#/fleet", icon: WrenchIcon },
];

const route = $derived(router.current);
// The fleet manager is offered only where fleet control is set up. Its page
// still explains the setup to anyone who opens it directly.
const access = createQuery(() => serviceAccessOptions);
const fleet = createQuery(() => fleetCatalogOptions(access.data !== undefined));
const managed = $derived(
  manage.filter(
    (row) => row.id !== "fleet" || fleet.data?.enabled === true || route.page === "fleet",
  ),
);
const query = $derived(
  route.page === "search" ? (new URLSearchParams(route.params).get("q") ?? "") : "",
);
const area = $derived.by((): string => {
  switch (route.page) {
    case "search":
      return query ? "search" : "chats";
    case "history":
    case "chat":
      return "chats";
    case "agent":
      return "agents";
    case "machine":
      return "machines";
    case "record":
    case "records":
      return "memory";
    case "setup":
      return "settings";
    default:
      return route.page;
  }
});
// Chats keeps its own pages under it: the recent list and what was read lately.
const child = $derived(
  route.page === "history" ? "history" : route.page === "search" && !query ? "recent" : "",
);
let chatsOpen = $state(false);
let arrived = "";
$effect(() => {
  if (area === arrived) return;
  arrived = area;
  chatsOpen = area === "chats";
});
// The chevron beside Chats is for pointers; keyboards open and close its
// pages from the row's own link with the arrow keys.
function arrows(event: KeyboardEvent): void {
  if (event.key === "ArrowRight" && !chatsOpen) chatsOpen = true;
  else if (event.key === "ArrowLeft" && chatsOpen) chatsOpen = false;
  else return;
  event.preventDefault();
}
function find(): void {
  close();
  askForSearch();
}

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

// Something above the rail, the top bar and the demo's banner, pushes it
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
const widths = { least: 180, usual: 220, most: 440 } as const;
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
// The top bar's left cell takes the same width in the Mac app, where it holds
// the traffic lights over the sidebar's colour (terminal.css, data-shell).
$effect(() => {
  document.documentElement.style.setProperty("--rail-width", `${width}px`);
});
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
</script>

{#snippet row(item: Row)}
  {@const on = item.id === area && !child}
  {@const count = counts[item.id]}
  <!-- The count is drawn beside the name but kept out of it, so the row is
       named by its section alone; the page it leads to says the number. -->
  <div class="nav-row" class:on>
    {#if item.id === "search"}
      <!-- Search is the field in the top bar; this row shows and focuses it. -->
      <button class="nav-link" type="button" aria-current={on ? "page" : undefined} aria-label="Search every chat" aria-keyshortcuts="Control+K Meta+K" onclick={find}><span class="nav-ico"><Icon icon={item.icon} size={15} state={on ? "on" : "rest"} /></span><span class="nav-label">{item.label}</span></button>
    {:else}
      <a class="nav-link" href={item.href} aria-current={on ? "page" : undefined} aria-keyshortcuts={item.id === "chats" ? "ArrowRight ArrowLeft" : undefined} onclick={close} onkeydown={item.id === "chats" ? arrows : undefined}><span class="nav-ico"><Icon icon={item.icon} size={15} state={on ? "on" : "rest"} /></span><span class="nav-label">{item.label}</span>{#if count !== undefined}<span class="nav-count" aria-hidden="true">{count.toLocaleString()}</span>{/if}</a>
    {/if}
    {#if item.id === "chats"}
      <button class="chev" type="button" tabindex="-1" aria-label="{chatsOpen ? 'Collapse' : 'Expand'} Chats" aria-expanded={chatsOpen} onclick={() => (chatsOpen = !chatsOpen)}><Icon icon={CaretRightIcon} size={14} /></button>
    {/if}
  </div>
  {#if item.id === "chats" && chatsOpen}
    <div class="subs">
      <a class="sub" class:on={child === "recent"} href="#/search" aria-current={child === "recent" ? "page" : undefined} onclick={close}><Icon icon={ClockIcon} size={14} state={child === "recent" ? "on" : "rest"} /><span>Recent chats</span></a>
      <a class="sub" class:on={child === "history"} href="#/history" aria-current={child === "history" ? "page" : undefined} onclick={close}><Icon icon={ClockCounterClockwiseIcon} size={14} state={child === "history" ? "on" : "rest"} /><span>Viewing history</span></a>
    </div>
  {/if}
{/snippet}

<aside id="rail" class:open style:--rail-width="{width}px" bind:this={aside}>
  <div class="rail-in" style:--rail-pushed="{pushed}px">
  <div class="sheet-top">
    <a class="brand" href="#/" onclick={close}><img src={asset("mash-mark.svg")} alt="" width="24" height="24" /><span>mash<i class="pat" aria-hidden="true"></i></span></a>
    <button id="xdrawer" type="button" aria-label="Close navigation" onclick={close}><Icon icon={XIcon} size={18} /></button>
  </div>
  {#if !catalogue.error}<div class="sheet-viewing"><ViewingSwitcher place="sheet" /></div>{/if}
  <nav class="nav" class:fade-above={fade.above} class:fade-below={fade.below} aria-label="Main" bind:this={nav} onscroll={measure}>
    {#each main as item (item.id)}{@render row(item)}{/each}
    <p class="nav-group">Manage</p>
    {#each managed as item (item.id)}{@render row(item)}{/each}
    <!-- The heading leads to every project; the rows are the pinned ones,
         then recent others while there is room. -->
    <a class="nav-group" href="#/projects" aria-current={route.page === "projects" ? "page" : undefined} onclick={close}>{allPinned ? "Pinned projects" : "Projects"}</a>
    {#each projects as project (project.key)}
      {@const on = route.page === "project" && route.name === project.key}
      <div class="nav-row" class:on>
        <a class="nav-link" href={projectHref(project.key)} title={project.key} aria-current={on ? "page" : undefined} onclick={close}><span class="nav-ico">{#if choices.isPinned(project.key)}<Icon icon={PushPinIcon} size={15} state="on" aria-label="Pinned" />{:else}<Icon icon={FolderIcon} size={15} state={on ? "on" : "rest"} />{/if}</span><span class="nav-label">{projectLabel(project.key)}</span></a>
      </div>
    {/each}
    <RailFoot unavailable={catalogue.error !== null} />
  </nav>
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div id="rail-grip" class:on={dragging} role="separator" aria-orientation="vertical" aria-label="Resize navigation" aria-valuemin={widths.least} aria-valuemax={widths.most} aria-valuenow={width} tabindex="0" onpointerdown={grip} onkeydown={nudge} ondblclick={() => resize(widths.usual)}></div>
</aside>

<style>
  /* The sidebar's labels are controls, not text: a drag across them selects nothing. */
  #rail {
    -webkit-user-select: none;
    user-select: none;
  }
</style>
