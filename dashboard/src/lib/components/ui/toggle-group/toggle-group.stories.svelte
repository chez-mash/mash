<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../../../../stories/Case.svelte";
import Cases from "../../../../stories/Cases.svelte";
import * as ToggleGroup from "./index";

// The segmented control: a muted track, the chosen segment raised.
const { Story } = defineMeta({ title: "Kit/ToggleGroup", component: ToggleGroup.Root });
</script>

<Story name="Two choices" asChild>
  <ToggleGroup.Root type="single" aria-label="History range" value="recent">
    <ToggleGroup.Item value="all">all history</ToggleGroup.Item>
    <ToggleGroup.Item value="recent">recent updates</ToggleGroup.Item>
  </ToggleGroup.Root>
</Story>

<Story name="States" asChild>
  <Cases>
    <Case label="one chosen">
      <ToggleGroup.Root type="single" aria-label="Theme" value="system">
        <ToggleGroup.Item value="system">match the system</ToggleGroup.Item>
        <ToggleGroup.Item value="light">light</ToggleGroup.Item>
        <ToggleGroup.Item value="dark">dark</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
    <Case label="none chosen">
      <ToggleGroup.Root type="single" aria-label="Theme, none chosen">
        <ToggleGroup.Item value="system">match the system</ToggleGroup.Item>
        <ToggleGroup.Item value="light">light</ToggleGroup.Item>
        <ToggleGroup.Item value="dark">dark</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
    <Case label="disabled">
      <ToggleGroup.Root type="single" aria-label="Index into, disabled" value="both" disabled>
        <ToggleGroup.Item value="both">this machine and the shared server</ToggleGroup.Item>
        <ToggleGroup.Item value="local">this machine only</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
    <Case label="one segment disabled">
      <ToggleGroup.Root type="single" aria-label="Index into, one disabled" value="local">
        <ToggleGroup.Item value="both" disabled>this machine and the shared server</ToggleGroup.Item>
        <ToggleGroup.Item value="local">this machine only</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
  </Cases>
</Story>

<Story name="Focused" play={async ({ canvas }) => { canvas.getByRole("radio", { name: "light" }).focus(); }} asChild>
  <ToggleGroup.Root type="single" aria-label="Theme" value="system">
    <ToggleGroup.Item value="system">match the system</ToggleGroup.Item>
    <ToggleGroup.Item value="light">light</ToggleGroup.Item>
    <ToggleGroup.Item value="dark">dark</ToggleGroup.Item>
  </ToggleGroup.Root>
</Story>
