<script lang="ts">
import { agents } from "../lib/agents";
import type { Agent } from "../lib/types";

// An agent's logo in its brand colour on a small ivory tile (see .mark in
// terminal.css). The logo is inline SVG; lib/agents.ts says why.
let { agent, size = "md" }: { agent: Agent; size?: "sm" | "md" | "lg" } = $props();
</script>

<span class="mark" class:sm={size === "sm"} class:lg={size === "lg"} aria-hidden="true" style:color={agents[agent].tint}>{@html agents[agent].logo}</span>
