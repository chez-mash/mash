<script lang="ts">
// Where a chat happened, as icons with words: the agent, the machine and the
// project. The icons take the place of the separators a plain line needs, so
// each name keeps its text and the icon says which kind of name it is. The
// role is in each name's tooltip; the names themselves stay readable text.
import Bot from "@lucide/svelte/icons/bot";
import Folder from "@lucide/svelte/icons/folder";
import Laptop from "@lucide/svelte/icons/laptop";
import { agents } from "../lib/agents";
import { projectLabel } from "../lib/browse";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import MachineIcon from "./MachineIcon.svelte";

let {
  agent,
  machine = "",
  project = "",
  mark = false,
  os = false,
  size = 12,
}: {
  agent?: Agent | undefined;
  machine?: string;
  /** The project's path; its last part is shown. */
  project?: string;
  /** Show the agent's own mark; rows that already lead with it use a plain icon. */
  mark?: boolean;
  /** Draw the machine's system logo. Those logos come from the machine
   * checks, which repeat every minute while one is on screen, so only a page
   * that already shows the machines' state asks for them; elsewhere a plain
   * laptop keeps the page from reading anything in the background. */
  os?: boolean;
  size?: number;
} = $props();
</script>

<span class="cpl">
  {#if agent}
    <span class="cpl-i" title="Agent: {agents[agent].name}">{#if mark}<AgentMark {agent} size="sm" />{:else}<Bot {size} aria-hidden="true" />{/if}{agents[agent].name}</span>
  {/if}
  {#if machine}
    <span class="cpl-i" title="Machine: {machine}"><span class="cpl-g" aria-hidden="true">{#if os}<MachineIcon {machine} {size} />{:else}<Laptop {size} />{/if}</span>{machine}</span>
  {/if}
  {#if project}
    <span class="cpl-i" title="Project: {project}"><Folder {size} aria-hidden="true" />{projectLabel(project)}</span>
  {/if}
</span>

<style>
  .cpl {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 2px 10px;
    min-width: 0;
  }
  .cpl-i {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    white-space: nowrap;
  }
  .cpl-i > :global(svg),
  .cpl-g {
    flex: none;
    display: inline-flex;
  }
  .cpl-i > :global(.mark.sm) {
    flex: none;
    width: 14px;
    height: 14px;
  }
</style>
