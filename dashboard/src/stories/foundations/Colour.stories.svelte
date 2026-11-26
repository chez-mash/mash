<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import { grouped, isColour, roles } from "../tokens";

// Every colour role of the theme as a swatch under its name. The toolbar's
// theme switch shows the same roles in dark.
const { Story } = defineMeta({ title: "Foundations/Colour" });

const colours = grouped(roles().filter((role) => isColour(role.name)));
</script>

<Story name="Roles" asChild>
  <div class="groups">
    {#each colours as group (group.family)}
      <section>
        <h2>{group.family}</h2>
        <div class="swatches">
          {#each group.roles as role (role.name)}
            <div class="swatch">
              <span class="chip" style:background="var({role.name})"></span>
              <code>{role.name}</code>
            </div>
          {/each}
        </div>
      </section>
    {/each}
  </div>
</Story>

<style>
  .groups {
    display: flex;
    flex-direction: column;
    gap: 22px;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 12px;
    font-weight: var(--weight-strong);
    color: var(--text-muted);
    text-transform: capitalize;
  }
  .swatches {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 10px 14px;
  }
  .swatch {
    display: flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
  }
  /* The swatch sits on the page's surface with a hairline, so a tint or a
     near-white role still shows its edge. */
  .chip {
    flex: none;
    width: 34px;
    height: 34px;
    border: var(--border-fill);
    border-radius: var(--radius-5);
  }
  code {
    min-width: 0;
    font-family: var(--type-mono);
    font-size: 11.5px;
    color: var(--text-body);
    overflow-wrap: anywhere;
  }
</style>
