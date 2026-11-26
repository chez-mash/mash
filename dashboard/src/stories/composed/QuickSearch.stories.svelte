<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import QuickSearch from "../../components/QuickSearch.svelte";

// The quick panel the desktop app shows from the menu bar: search alone, in
// the system's look.
const { Story } = defineMeta({
  title: "Composed/QuickSearch",
  component: QuickSearch,
  parameters: { layout: "fullscreen", data: { route: "#/quick", surface: "native" } },
  tags: ["screen"],
});
</script>

<Story name="Empty" />

<Story
  name="Typed"
  play={async ({ canvas, userEvent }) => {
    await userEvent.type(canvas.getByPlaceholderText("Search Mash"), "checkout");
    await canvas.findAllByText(/flaky checkout test/);
  }}
/>

<Story
  name="Actions open"
  play={async ({ canvas, userEvent }) => {
    await userEvent.type(canvas.getByPlaceholderText("Search Mash"), "checkout");
    await canvas.findAllByText(/flaky checkout test/);
    await userEvent.click(canvas.getByRole("button", { name: "Actions" }));
    await canvas.findByRole("menu", { name: "Actions" });
  }}
/>
