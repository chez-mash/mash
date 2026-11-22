<script lang="ts">
import { agents } from "../lib/agents";
import type { Agent } from "../lib/types";

// Provider marks are single-colour drawings, drawn inside a small tile in the
// agent's colour (see .mark.tint), so agents can be told apart at a glance.
let { agent, size = 19 }: { agent: Agent; size?: number } = $props();
const info = $derived(agents[agent]);
</script>

<span class="mark tint" aria-hidden="true" style:width="{size}px" style:height="{size}px" style:--mark-colour={info.tint} style:--mark-logo="url({info.logo})"></span>
