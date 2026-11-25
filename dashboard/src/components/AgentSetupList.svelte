<script lang="ts">
import Bot from "@lucide/svelte/icons/bot";
import type { Snippet } from "svelte";
import MachineSetup from "./MachineSetup.svelte";

// One machine's agent setup as a section: its heading, the list of agents
// part by part, and what Mash does to find out. `action` sits beside the
// heading, such as the way to connect another agent.
let {
  machineId,
  enabled,
  title = "Agent setup",
  note = "Mash reads settings and files for this; it does not start the agents.",
  action,
}: {
  machineId: string;
  enabled: boolean;
  title?: string;
  note?: string;
  action?: Snippet;
} = $props();
</script>

<section class="sec">
  <div class="hd"><h2 class="h2"><Bot size={15} aria-hidden="true" />{title}</h2>{#if action}{@render action()}{/if}</div>
  <div class="rws agentset solo">
    <div class="rw head" aria-hidden="true"><span>Agent</span><span>Parts</span><span>Status</span></div>
    <MachineSetup {machineId} {enabled} showMachine={false} />
  </div>
  {#if note}<p class="meta">{note}</p>{/if}
</section>

