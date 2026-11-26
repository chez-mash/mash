<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import Icon from "$lib/components/Icon.svelte";
import { controlled } from "../stories/fixtures";
import ConfirmCard from "./ConfirmCard.svelte";
import ServiceList from "./ServiceList.svelte";

// One machine's services as rows with the controls the machine offers. A
// control only asks; the page confirms under the row before anything is sent.
const { Story } = defineMeta({ title: "Blocks/ServiceList", component: ServiceList });

const mixed = controlled({ watcher: "healthy", qdrant: "stopped", mcp: "failed" });
const running = controlled({ watcher: "healthy", qdrant: "healthy", mcp: "healthy" });
const unreachable = { ...running, reachable: false };
const ask = (): void => {};
</script>

{#snippet none()}{/snippet}

<Story name="Running, stopped and failed" asChild>
  <ServiceList machine={mixed} disabled={false} {ask} confirmation={none} />
</Story>

<Story name="All running" asChild>
  <ServiceList machine={running} disabled={false} {ask} confirmation={none} />
</Story>

<Story name="Controls disabled" asChild>
  <ServiceList machine={mixed} disabled {ask} confirmation={none} />
</Story>

<Story name="Machine unreachable" asChild>
  <ServiceList machine={unreachable} disabled={false} {ask} confirmation={none} />
</Story>

<Story name="Asking before a stop" asChild>
  <ServiceList machine={running} disabled={false} {ask}>
    {#snippet confirmation(service)}
      {#if service.kind === "watcher"}
        <ConfirmCard label="Confirm service action">
          <span>Stop {service.name} on laptop? New chats may stop appearing until the watcher is running again. Existing indexed history is retained.</span>
          <button class="btn p" type="button"><Icon icon={CheckIcon} size={14} state="strong" />Confirm stop</button>
          <button class="btn" type="button">Cancel</button>
        </ConfirmCard>
      {/if}
    {/snippet}
  </ServiceList>
</Story>

<Story name="On a phone" tags={["phone"]} globals={{ viewport: { value: "phone" } }} asChild>
  <ServiceList machine={mixed} disabled={false} {ask} confirmation={none} />
</Story>
