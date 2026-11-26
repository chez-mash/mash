<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Icon from "$lib/components/Icon.svelte";
import KindIcon from "../../components/KindIcon.svelte";
import KindPill from "../../components/KindPill.svelte";
import { kindIcons } from "../../lib/recordIcons";
import { kinds } from "../../lib/records";

// A memory's kind keeps one tint wherever it is shown: butter for a decision,
// blue for a preference, teal for a note; on the tile, the pill and the icon,
// always beside the kind's own glyph.
const { Story } = defineMeta({ title: "Foundations/Memory kinds" });
</script>

<Story name="Tints" asChild>
  <table>
    <thead>
      <tr><th>kind</th><th>tile</th><th>pill</th><th>icon</th><th>roles</th></tr>
    </thead>
    <tbody>
      {#each kinds as kind (kind)}
        <tr>
          <th>{kind}</th>
          <td><span class="tile kind-{kind}" aria-hidden="true"><Icon icon={kindIcons[kind]} size={15} /></span></td>
          <td><KindPill {kind} /></td>
          <td><KindIcon {kind} size={16} /></td>
          <td><code>--kind-{kind}-bg</code> <code>--kind-{kind}-text</code></td>
        </tr>
      {/each}
    </tbody>
  </table>
</Story>

<style>
  table {
    border-collapse: collapse;
  }
  th,
  td {
    padding: 10px 26px 10px 0;
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
  /* The tile a memory's row leads with (MemoryItem). */
  .tile {
    display: inline-grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-5);
  }
</style>
