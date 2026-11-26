<script lang="ts">
import type { ServiceState } from "../lib/services.svelte";
import { stateLabel } from "../lib/services.svelte";
import { serviceTone } from "../lib/tone";
import ToneIcon from "./ToneIcon.svelte";

let { state, label }: { state: ServiceState; label?: string } = $props();
// A pill whose fill and icon say the state: a green check when healthy, an
// amber warning when it needs attention, a red cross when it failed, and a
// grey question mark when the state is unknown.
const tone = $derived(serviceTone(state));
</script>

<span><span class="pill svc-pill st tone-{tone}" title={label ?? stateLabel[state]}><ToneIcon {tone} size={12} /><span>{label ?? stateLabel[state]}</span></span></span>
