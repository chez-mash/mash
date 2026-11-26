<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import { type Tone, toneIcons } from "../lib/tone";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import ToneIcon from "./ToneIcon.svelte";

const { Story } = defineMeta({
  title: "Blocks/ToneIcon",
  component: ToneIcon,
  args: { tone: "ok" },
});

const tones = Object.keys(toneIcons) as Tone[];
</script>

<Story name="Default" />

<Story name="Tones" asChild>
  <Cases row>
    {#each tones as tone (tone)}
      <Case label={tone}><ToneIcon {tone} size={16} /></Case>
    {/each}
  </Cases>
</Story>

<Story name="Sizes" asChild>
  <Cases row>
    {#each [12, 13, 14, 16, 20] as size (size)}
      <Case label="{size}px"><ToneIcon tone="warn" {size} /> <ToneIcon tone="ok" {size} /></Case>
    {/each}
  </Cases>
</Story>

<!-- Alone, the icon carries its state's name for screen readers and a hint. -->
<Story name="Labelled" args={{ tone: "bad", label: "Failed" }} />
