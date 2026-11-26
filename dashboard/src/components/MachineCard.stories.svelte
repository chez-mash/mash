<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import type { ComponentProps } from "svelte";
import { groupChats } from "../lib/browse";
import type { StoryData } from "../stories/answers";
import { catalog, demoAnswer, required, wireMachine } from "../stories/fixtures";
import Narrow from "../stories/Narrow.svelte";
import MachineCard from "./MachineCard.svelte";

// One machine as a card. What it says comes from the dashboard's own service
// checks, so each story answers that check differently.
const groups = groupChats(catalog.all, (chat) => chat.machine);
const group = required(groups.find((entry) => entry.key === "laptop") ?? groups[0], "machine");

const checks = (machines: ReturnType<typeof wireMachine>[]): { data: StoryData } => ({
  data: { answers: { "/api/services": { ...demoAnswer("/api/services"), machines } } },
});

const { Story } = defineMeta({
  title: "Blocks/MachineCard",
  component: MachineCard,
  args: { group, local: "laptop" },
  render: narrow,
});
</script>

{#snippet narrow(args: ComponentProps<typeof MachineCard>)}
  <Narrow width={300}><MachineCard {...args} /></Narrow>
{/snippet}

<Story name="Healthy" parameters={checks([wireMachine({ id: group.key })])} />

<Story
  name="Needs a look"
  parameters={checks([
    wireMachine({ id: group.key }, (service) =>
      service.kind === "watcher" ? { ...service, state: "degraded", summary: "Behind · 14 chats waiting" } : service,
    ),
  ])}
/>

<Story
  name="Failed"
  parameters={checks([
    wireMachine({ id: group.key }, (service) =>
      service.kind === "qdrant" ? { ...service, state: "failed", summary: "Exited with status 1" } : service,
    ),
  ])}
/>

<Story name="Offline" parameters={checks([wireMachine({ id: group.key, reachable: false })])} />

<!-- Nothing checks this machine and no server reports on it. -->
<Story name="No status" parameters={checks([])} />

<Story name="Another machine" args={{ local: "workstation" }} parameters={checks([wireMachine({ id: group.key, platform: "macos" })])} />
