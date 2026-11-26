<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import { roles } from "../tokens";

// Corners and shadows: the one radius scale components pick a step from, and
// the shadows that lift a panel, a menu or a knob off the page.
const { Story } = defineMeta({ title: "Foundations/Shape" });

const all = roles();
const radii = all.filter((role) => role.name.startsWith("--radius-"));
const shadows = all.filter((role) => role.name.startsWith("--shadow-"));
const rings = ["--memory-ring", "--selected-ring", "--focus"];
</script>

<Story name="Radii" asChild>
  <div class="grid">
    {#each radii as role (role.name)}
      <div class="cell"><span class="box" style:border-radius="var({role.name})"></span><code>{role.name}</code><small>{role.written}</small></div>
    {/each}
  </div>
</Story>

<Story name="Shadows" asChild>
  <div class="grid wide">
    {#each shadows as role (role.name)}
      <div class="cell"><span class="box raised" style:box-shadow="var({role.name})"></span><code>{role.name}</code></div>
    {/each}
    <div class="cell"><span class="box raised" style:box-shadow="0 0 0 1px var(--memory-ring), var(--memory-glow)"></span><code>--memory-ring + --memory-glow</code></div>
    <div class="cell"><span class="box raised" style:box-shadow="var(--find-glow)"></span><code>--find-glow</code></div>
  </div>
</Story>

<Story name="Rings" asChild>
  <div class="grid wide">
    {#each rings as ring (ring)}
      <div class="cell"><span class="box raised" style:outline="2px solid var({ring})"></span><code>{ring}</code></div>
    {/each}
  </div>
</Story>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 22px 16px;
  }
  .grid.wide {
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 34px 26px;
    padding: 10px;
  }
  .cell {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .box {
    width: 84px;
    height: 56px;
    border: var(--border-strong);
    background: var(--surface-muted);
  }
  .box.raised {
    border: 0;
    border-radius: var(--radius-7);
    background: var(--surface-raised);
    margin-bottom: 8px;
  }
  code {
    font-family: var(--type-mono);
    font-size: 11.5px;
    color: var(--text-body);
  }
  small {
    font-size: 11.5px;
    color: var(--text-faint);
  }
</style>
