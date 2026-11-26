<script lang="ts">
// A memory's kind as a pill in the kind's own tint, with its icon: butter for
// a decision, blue for a preference, teal for a note.
import Bookmark from "@lucide/svelte/icons/bookmark";
import { kindIcons } from "../lib/recordIcons";
import { label } from "../lib/records";

let { kind, lower = false }: { kind: string; lower?: boolean } = $props();
const Icon = $derived(kindIcons[kind as keyof typeof kindIcons] ?? Bookmark);
const words = $derived(lower ? label(kind).toLowerCase() : label(kind));
</script>

<span class="pill kind-{kind}"><Icon size={12} aria-hidden="true" />{words}</span>
