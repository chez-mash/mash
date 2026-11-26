<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import { wireMachine } from "../stories/fixtures";
import MachineIcon from "./MachineIcon.svelte";

// The system's mark when the dashboard knows what a machine runs, the plain
// laptop otherwise. What it knows comes from the service checks.
const { Story } = defineMeta({
  title: "Blocks/MachineIcon",
  component: MachineIcon,
  args: { machine: "laptop" },
  parameters: {
    data: {
      answers: {
        "/api/services": {
          machines: [
            wireMachine({ id: "laptop", platform: "linux" }),
            wireMachine({ id: "studio", platform: "macos" }),
            wireMachine({ id: "tower", platform: "windows" }),
          ],
        },
      },
    },
  },
});

const machines = [
  { machine: "laptop", label: "Linux" },
  { machine: "studio", label: "macOS" },
  { machine: "tower", label: "Windows" },
  { machine: "somewhere", label: "system not known" },
];
</script>

<Story name="Default" />

<Story name="Systems" asChild>
  <Cases row>
    {#each machines as entry (entry.machine)}
      <Case label={entry.label}><MachineIcon machine={entry.machine} /></Case>
    {/each}
  </Cases>
</Story>

<Story name="Sizes" asChild>
  <Cases row>
    {#each [12, 14, 16, 20] as size (size)}
      <Case label="{size}px"><MachineIcon machine="studio" {size} /> <MachineIcon machine="somewhere" {size} /></Case>
    {/each}
  </Cases>
</Story>
