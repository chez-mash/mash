<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";

// The typefaces, the sizes the dashboard sets text in, and the weights Geist
// ships at. Sizes are not roles: these are the steps the stylesheets use.
const { Story } = defineMeta({ title: "Foundations/Type" });

const sizes = [34, 26, 18, 16, 15, 14, 13.5, 13, 12.5, 12, 11.5, 11];
const weights = [
  { role: "--weight-regular", use: "body text" },
  { role: "--weight-label", use: "labels, navigation, row titles" },
  { role: "--weight-strong", use: "headings" },
  { role: "--weight-meta", use: "small muted text (heavier in dark)" },
];
const faces = [
  { role: "--type-sans", sample: "Fix the flaky checkout test" },
  { role: "--type-display", sample: "Good evening, Sam" },
  { role: "--type-mono", sample: "mash search --live checkout" },
];
const named = [
  { name: ".h1", tag: "h1", sample: "A page's title" },
  { name: ".h2", tag: "h2", sample: "A section's heading" },
  { name: ".meta", tag: "p", sample: "A quiet line under a title" },
  { name: ".eyebrow", tag: "p", sample: "An eyebrow" },
  { name: ".mono", tag: "p", sample: "PID 4821 · checked 2m ago" },
];
</script>

<Story name="Scale" asChild>
  <div class="rows">
    {#each sizes as size (size)}
      <div class="row"><code>{size}px</code><span style:font-size="{size}px">Sign-in uses fifteen-minute access tokens</span></div>
    {/each}
  </div>
</Story>

<Story name="Weights" asChild>
  <div class="rows">
    {#each weights as weight (weight.role)}
      <div class="row"><code>{weight.role}</code><span style:font-weight="var({weight.role})">Invoice exports run as background jobs</span><small>{weight.use}</small></div>
    {/each}
  </div>
</Story>

<Story name="Typefaces" asChild>
  <div class="rows">
    {#each faces as face (face.role)}
      <div class="row"><code>{face.role}</code><span class="face" style:font-family="var({face.role})">{face.sample}</span></div>
    {/each}
  </div>
</Story>

<Story name="Named styles" asChild>
  <div class="rows">
    {#each named as style (style.name)}
      <div class="row"><code>{style.name}</code><svelte:element this={style.tag} class={style.name.slice(1)}>{style.sample}</svelte:element></div>
    {/each}
  </div>
</Story>

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .row {
    display: grid;
    grid-template-columns: 150px minmax(0, auto) 1fr;
    align-items: baseline;
    gap: 16px;
    color: var(--text-strong);
  }
  .row > :global(:nth-child(2)) {
    margin: 0;
  }
  code {
    font-family: var(--type-mono);
    font-size: 11.5px;
    color: var(--text-faint);
  }
  small {
    font-size: 12px;
    color: var(--text-muted);
  }
  .face {
    font-size: 18px;
  }
</style>
