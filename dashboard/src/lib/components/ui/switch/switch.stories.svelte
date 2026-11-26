<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../../../../stories/Case.svelte";
import Cases from "../../../../stories/Cases.svelte";
import { Switch } from "./index";

const { Story } = defineMeta({
  title: "Kit/Switch",
  component: Switch,
  args: { "aria-label": "Index Codex on this machine" },
});
</script>

<Story name="Off" />

<Story name="On" args={{ checked: true }} />

<Story name="States" asChild>
  <Cases row>
    <Case label="off"><Switch aria-label="Off" /></Case>
    <Case label="on"><Switch aria-label="On" checked /></Case>
    <Case label="off, disabled"><Switch aria-label="Off, disabled" disabled /></Case>
    <Case label="on, disabled"><Switch aria-label="On, disabled" checked disabled /></Case>
  </Cases>
</Story>

<Story
  name="Focused"
  play={async ({ canvas }) => {
    canvas.getByRole("switch").focus();
  }}
/>
