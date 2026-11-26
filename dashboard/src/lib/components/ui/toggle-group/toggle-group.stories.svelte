<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import DesktopIcon from "phosphor-svelte/lib/DesktopIcon";
import MoonIcon from "phosphor-svelte/lib/MoonIcon";
import SunIcon from "phosphor-svelte/lib/SunIcon";
import Icon from "$lib/components/Icon.svelte";
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

<!-- Choosing another segment slides the raised highlight across to it, and
     resizes it on the way; with less motion it switches at once. -->
<Story name="Sliding highlight" asChild>
  <Cases>
    <Case label="two choices">
      <ToggleGroup.Root type="single" aria-label="History range, two" value="all">
        <ToggleGroup.Item value="all">all history</ToggleGroup.Item>
        <ToggleGroup.Item value="recent">recent updates</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
    <Case label="three choices">
      <ToggleGroup.Root type="single" aria-label="Theme, three" value="light">
        <ToggleGroup.Item value="system">match the system</ToggleGroup.Item>
        <ToggleGroup.Item value="light">light</ToggleGroup.Item>
        <ToggleGroup.Item value="dark">dark</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
    <Case label="five choices">
      <ToggleGroup.Root type="single" aria-label="Kind, five" value="decision">
        <ToggleGroup.Item value="all">all</ToggleGroup.Item>
        <ToggleGroup.Item value="decision">decisions</ToggleGroup.Item>
        <ToggleGroup.Item value="fact">facts</ToggleGroup.Item>
        <ToggleGroup.Item value="preference">preferences</ToggleGroup.Item>
        <ToggleGroup.Item value="task">tasks</ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
    <Case label="icons only">
      <ToggleGroup.Root type="single" aria-label="Theme, icons" value="dark">
        <ToggleGroup.Item value="system" aria-label="match the system"><Icon icon={DesktopIcon} size={16} /></ToggleGroup.Item>
        <ToggleGroup.Item value="light" aria-label="light"><Icon icon={SunIcon} size={16} /></ToggleGroup.Item>
        <ToggleGroup.Item value="dark" aria-label="dark"><Icon icon={MoonIcon} size={16} /></ToggleGroup.Item>
      </ToggleGroup.Root>
    </Case>
    <Case label="wrapped onto rows" note="no slide: each segment draws its own highlight">
      <div class="narrow">
        <ToggleGroup.Root type="single" aria-label="Index into, wrapped" value="local">
          <ToggleGroup.Item value="both">this machine and the shared server</ToggleGroup.Item>
          <ToggleGroup.Item value="local">this machine only</ToggleGroup.Item>
        </ToggleGroup.Root>
      </div>
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

<style>
  .narrow {
    width: 260px;
  }
</style>
