<script lang="ts">
import { agents } from "../lib/agents";
import type { Agent } from "../lib/types";

// An agent's logo on a small tile in its brand colour, as Raycast shows app
// icons (see .mark in terminal.css). The logo is inline SVG; lib/agents.ts says why.
let { agent, size = "md" }: { agent: Agent; size?: "sm" | "md" | "lg" } = $props();
</script>

<span class="mark" class:sm={size === "sm"} class:lg={size === "lg"} aria-hidden="true" style:background={agents[agent].tint} style:color={agents[agent].ink}>{@html agents[agent].logo}</span>
