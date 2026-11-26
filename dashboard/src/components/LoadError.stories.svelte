<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import LoadError from "./LoadError.svelte";

const retry = (): void => {};

const { Story } = defineMeta({
  title: "Blocks/LoadError",
  component: LoadError,
  args: {
    title: "Couldn’t load chats",
    message: "The index is being rebuilt. Try again in a moment.",
    retry,
  },
});
</script>

<Story name="Default" />

<Story name="States" asChild>
  <Cases width={720}>
    <Case label="what the dashboard said"><LoadError title="Couldn’t load chats" message="The index is being rebuilt. Try again in a moment." {retry} /></Case>
    <Case label="could not reach Mash" note="a browser's own words are replaced"><LoadError title="Couldn’t load memory" message="Failed to fetch" {retry} /></Case>
    <Case label="retrying"><LoadError title="Couldn’t load chats" message="The index is being rebuilt. Try again in a moment." {retry} busy /></Case>
    <Case label="another word for retry"><LoadError title="Couldn’t connect to services" message="This dashboard has no services configuration." {retry} retryLabel="Check again" /></Case>
  </Cases>
</Story>

<Story name="On a phone" tags={["phone"]} globals={{ viewport: { value: "phone" } }} />
