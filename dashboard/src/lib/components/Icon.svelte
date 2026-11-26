<script lang="ts">
import type { IconComponentProps } from "phosphor-svelte";
import type { Component } from "svelte";
import { type IconState, type IconWeight, iconTheme, iconWeight } from "$lib/icon.svelte";
import { glyphName } from "$lib/iconGlyphs";

// Every interface icon is drawn through this: a Phosphor glyph at a size, in
// the weight lib/icon.svelte.ts gives that size and state. It takes the text
// colour around it. An icon is decoration unless it is given an aria-label.
let {
  icon: Glyph,
  size = 16,
  state = "rest",
  weight,
  color,
  class: className,
  style,
  "aria-label": label,
}: {
  icon: Component<IconComponentProps>;
  size?: number;
  state?: IconState;
  /** A weight the rules do not give for this size and state. */
  weight?: IconWeight | undefined;
  /** A colour of its own; otherwise the text colour around it. */
  color?: string | undefined;
  class?: string | undefined;
  style?: string | undefined;
  "aria-label"?: string | undefined;
} = $props();

const drawn = $derived(weight ?? iconWeight(size, state, iconTheme.weights));
const name = $derived(glyphName(Glyph));
// Only what was given is passed on, so the glyph keeps its own defaults.
const given = $derived({
  ...(color === undefined ? {} : { color }),
  ...(style === undefined ? {} : { style }),
  ...(label === undefined ? { "aria-hidden": "true" as const } : { "aria-label": label }),
});
</script>

<Glyph {size} weight={drawn} class={[className, name && `ph-${name}`]} {...given} />
