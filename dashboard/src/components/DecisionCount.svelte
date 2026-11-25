<script lang="ts">
// How many decisions a chat produced, as the decision icon and a number. The
// words stay in the accessible text and the tooltip. The caller draws the
// pill around it.
import { kindIcons } from "../lib/recordIcons";

let { count }: { count: number } = $props();
const words = $derived(`${count.toLocaleString()} decision${count === 1 ? "" : "s"}`);
const Icon = kindIcons.decision;
</script>

<span class="dc" title={words}><Icon size={12} aria-hidden="true" />{count.toLocaleString()}<span class="vh">{count === 1 ? " decision" : " decisions"}</span></span>

<style>
  .dc {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .vh {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
