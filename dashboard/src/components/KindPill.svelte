<script lang="ts">
// A memory's kind as a pill in the kind's own tint, with its icon: butter for
// a decision, blue for a preference, teal for a note.
import BookmarkSimpleIcon from "phosphor-svelte/lib/BookmarkSimpleIcon";
import Icon from "$lib/components/Icon.svelte";
import { kindIcons } from "../lib/recordIcons";
import { label } from "../lib/records";

let { kind, lower = false }: { kind: string; lower?: boolean } = $props();
const Glyph = $derived(kindIcons[kind as keyof typeof kindIcons] ?? BookmarkSimpleIcon);
const words = $derived(lower ? label(kind).toLowerCase() : label(kind));
</script>

<span class="pill kind-{kind}"><Icon icon={Glyph} size={12} />{words}</span>
