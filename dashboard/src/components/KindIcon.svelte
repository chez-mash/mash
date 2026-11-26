<script lang="ts">
// A memory's kind as its icon, the same one the memory pages use. The kind's
// name stays in the accessible text and the tooltip, so the icon replaces the
// word without losing it. It takes the kind's own tint (see semantics.css).
import Bookmark from "@lucide/svelte/icons/bookmark";
import { kindIcons } from "../lib/recordIcons";
import { label } from "../lib/records";

let { kind, size = 13 }: { kind: string; size?: number } = $props();
const Icon = $derived(kindIcons[kind as keyof typeof kindIcons] ?? Bookmark);
</script>

<span class="ki ink-{kind}" title={label(kind)}><Icon {size} aria-hidden="true" /><span class="vh">{label(kind)}</span></span>

<style>
  .ki {
    display: inline-flex;
    flex: none;
    align-items: center;
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
