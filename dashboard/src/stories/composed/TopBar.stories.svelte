<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import TopBar from "../../components/TopBar.svelte";

// The bar across the top of every page. On a phone it keeps the mark, a
// search button and the menu button that opens the navigation sheet.
const { Story } = defineMeta({
  title: "Composed/TopBar",
  component: TopBar,
  args: { railOpen: false, toggleRail: () => {} },
  parameters: { layout: "fullscreen" },
  tags: ["screen"],
});
const phone = { viewport: { value: "phone" } };
</script>

<Story name="Desktop" />

<Story name="On the results page" parameters={{ data: { route: "#/search?q=checkout" } }} />

<Story name="Phone" tags={["phone"]} globals={phone} />

<Story name="Phone, searching" tags={["phone"]} globals={phone} parameters={{ data: { route: "#/search?q=checkout" } }} />
