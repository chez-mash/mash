<script lang="ts">
// A state as its icon and word in the tone's colour; with `pill`, on the
// tone's soft fill. The word is always shown: the colour and the shape only
// help it read at a glance.
import type { Tone } from "../lib/tone";
import ToneIcon from "./ToneIcon.svelte";

let {
  tone,
  words,
  pill = false,
  title,
  size = 13,
  class: className = "",
}: {
  tone: Tone;
  words: string;
  pill?: boolean;
  title?: string | undefined;
  size?: number;
  class?: string;
} = $props();
</script>

<span class="st tone-{tone} {className}" class:pill class:st-pill={pill} {title}><ToneIcon {tone} {size} /><span class="st-w">{words}</span></span>
