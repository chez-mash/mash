<script lang="ts">
// Where a chat happened, as marks with words: the agent's own tile, the
// machine's system logo and the project's folder. The marks take the place of
// the separators a plain line needs, so each name keeps its text and the mark
// says which kind of name it is. The role is in each name's tooltip; the names
// themselves stay readable text. A row that already leads with the agent's
// tile (`led`) names the agent without a second tile.
import Folder from "@lucide/svelte/icons/folder";
import { Tip } from "$lib/components/ui/tooltip";
import { agents } from "../lib/agents";
import { projectLabel } from "../lib/browse";
import type { Agent } from "../lib/types";
import AgentMark from "./AgentMark.svelte";
import MachineIcon from "./MachineIcon.svelte";

let {
  agent,
  machine = "",
  project = "",
  led = false,
  size = 12,
}: {
  agent?: Agent | undefined;
  machine?: string;
  /** The project's path; its last part is shown. */
  project?: string;
  /** The row already leads with the agent's tile. */
  led?: boolean;
  size?: number;
} = $props();
</script>

<span class="cpl">
  {#if agent}
    <Tip text="Agent: {agents[agent].name}" passive>{#snippet children(tip)}<span {...tip} class="cpl-i" data-place="agent">{#if !led}<AgentMark {agent} size="sm" />{/if}{agents[agent].name}</span>{/snippet}</Tip>
  {/if}
  {#if machine}
    <Tip text="Machine: {machine}" passive>{#snippet children(tip)}<span {...tip} class="cpl-i" data-place="machine"><span class="cpl-g" aria-hidden="true"><MachineIcon {machine} {size} /></span>{machine}</span>{/snippet}</Tip>
  {/if}
  {#if project}
    <Tip text="Project: {project}" passive>{#snippet children(tip)}<span {...tip} class="cpl-i" data-place="project"><Folder {size} aria-hidden="true" />{projectLabel(project)}</span>{/snippet}</Tip>
  {/if}
</span>

<style>
  .cpl {
    font-weight: var(--weight-meta);
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
