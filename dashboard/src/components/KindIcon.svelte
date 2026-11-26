<script lang="ts">
// A memory's kind as its icon, the same one the memory pages use. The kind's
// name stays in the accessible text and the tooltip, so the icon replaces the
// word without losing it. It takes the kind's own tint (see semantics.css).
import Bookmark from "@lucide/svelte/icons/bookmark";
import { Tip } from "$lib/components/ui/tooltip";
import { kindIcons } from "../lib/recordIcons";
import { label } from "../lib/records";

let { kind, size = 13 }: { kind: string; size?: number } = $props();
const Icon = $derived(kindIcons[kind as keyof typeof kindIcons] ?? Bookmark);
</script>

<Tip text={label(kind)} passive>{#snippet children(tip)}<span {...tip} class="ki ink-{kind}"><Icon {size} aria-hidden="true" /><span class="vh">{label(kind)}</span></span>{/snippet}</Tip>

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
