<script lang="ts">
import type { ServiceState } from "../lib/services.svelte";
import { stateLabel } from "../lib/services.svelte";

let { state, label }: { state: ServiceState; label?: string } = $props();
// A pill whose fill says the state: mint healthy, mustard needs attention,
// red failed, plain paper when the state is unknown.
const tone = {
  healthy: "ok",
  degraded: "warn",
  stopped: "warn",
  failed: "bad",
  unknown: "",
} as const;
</script>

<span><span class="pill svc-pill {tone[state]}" title={label ?? stateLabel[state]}><i class="dot" aria-hidden="true"></i><span>{label ?? stateLabel[state]}</span></span></span>
