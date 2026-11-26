<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Rail from "../../components/Rail.svelte";

// The navigation rail beside every page; a drawer over the page on a phone.
// The current item is filled, as `state="on"` draws it.
const { Story } = defineMeta({
  title: "Composed/Rail",
  component: Rail,
  parameters: { layout: "fullscreen" },
  tags: ["screen"],
});
const close = (): void => {};
const phone = { viewport: { value: "phone" } };
</script>

{#snippet rail(open: boolean)}
  <div id="frame">
    <Rail {open} {close} />
    <div id="col"><main id="main" tabindex="-1"><p class="meta">The page.</p></main></div>
  </div>
{/snippet}

<Story name="On the overview" asChild>{@render rail(false)}</Story>

<Story name="On memory" parameters={{ data: { route: "#/records" } }} asChild>{@render rail(false)}</Story>

<Story name="On a project" parameters={{ data: { route: `#/project?name=${encodeURIComponent("/home/sam/code/storefront")}` } }} asChild>
  {@render rail(false)}
</Story>

<Story name="Phone drawer" tags={["phone"]} globals={phone} parameters={{ data: { route: "#/machines" } }} asChild>
  {@render rail(true)}
</Story>
