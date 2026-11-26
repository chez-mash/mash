<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import BrainIcon from "phosphor-svelte/lib/BrainIcon";
import ChatsIcon from "phosphor-svelte/lib/ChatsIcon";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import GearSixIcon from "phosphor-svelte/lib/GearSixIcon";
import HardDrivesIcon from "phosphor-svelte/lib/HardDrivesIcon";
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
import WarningIcon from "phosphor-svelte/lib/WarningIcon";
import Icon from "$lib/components/Icon.svelte";
import { iconTheme, iconWeight, weightTokens } from "$lib/icon.svelte";

// The specimen: a set of glyphs at the sizes the dashboard draws, in each
// weight, then what the rules in lib/icon.svelte.ts choose. A component never
// sets a weight; it gives a size and, where it applies, a state.
const { Story } = defineMeta({ title: "Foundations/Icons", component: Icon });

const glyphs = [
  ChatsIcon,
  BrainIcon,
  FolderIcon,
  LaptopIcon,
  HardDrivesIcon,
  MagnifyingGlassIcon,
  GearSixIcon,
  WarningIcon,
];
const sizes = [14, 15, 16, 20, 28];
const weights = ["regular", "bold", "fill"] as const;
const states = [
  { state: "rest", use: "an ordinary icon: weighted by its size" },
  { state: "on", use: "the current item, the selected segment, anything switched on" },
  { state: "strong", use: "a warning, a failure, the icon on a primary button" },
] as const;
</script>

<Story name="Specimen" asChild>
  <table>
    <thead>
      <tr><th>size</th>{#each weights as weight (weight)}<th>{weight}</th>{/each}</tr>
    </thead>
    <tbody>
      {#each sizes as size (size)}
        <tr>
          <th>{size}px</th>
          {#each weights as weight (weight)}
            <td><span class="set">{#each glyphs as glyph, index (index)}<Icon icon={glyph} {size} {weight} />{/each}</span></td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</Story>

<Story name="By size" asChild>
  <table>
    <thead>
      <tr><th>size</th><th>the theme draws it</th><th></th></tr>
    </thead>
    <tbody>
      {#each sizes as size (size)}
        <tr>
          <th>{size}px</th>
          <td><code>{iconWeight(size, "rest", iconTheme.weights)}</code></td>
          <td><span class="set">{#each glyphs as glyph, index (index)}<Icon icon={glyph} {size} />{/each}</span></td>
        </tr>
      {/each}
    </tbody>
  </table>
</Story>

<Story name="States" asChild>
  <table>
    <thead>
      <tr><th>state</th><th>role</th><th>14px</th><th>16px</th><th>20px</th><th>used for</th></tr>
    </thead>
    <tbody>
      {#each states as entry (entry.state)}
        <tr>
          <th>{entry.state}</th>
          <td><code>{entry.state === "rest" ? "by size" : weightTokens[entry.state]}</code></td>
          {#each [14, 16, 20] as size (size)}
            <td><span class="set">{#each glyphs.slice(0, 4) as glyph, index (index)}<Icon icon={glyph} {size} state={entry.state} />{/each}</span></td>
          {/each}
          <td class="use">{entry.use}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</Story>

<style>
  table {
    border-collapse: collapse;
    color: var(--text-strong);
  }
  th,
  td {
    padding: 10px 22px 10px 0;
    text-align: left;
    vertical-align: middle;
  }
  th,
  code {
    font-family: var(--type-mono);
    font-size: 11.5px;
    font-weight: var(--weight-regular);
    color: var(--text-faint);
  }
  .set {
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }
  .use {
    font-size: 12.5px;
    color: var(--text-muted);
  }
</style>
