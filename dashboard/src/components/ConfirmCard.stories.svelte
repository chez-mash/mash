<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import CheckIcon from "phosphor-svelte/lib/CheckIcon";
import Icon from "$lib/components/Icon.svelte";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import ConfirmCard from "./ConfirmCard.svelte";

// The amber confirmation card: the question asked before a change is made.
const { Story } = defineMeta({ title: "Blocks/ConfirmCard", component: ConfirmCard });
</script>

{#snippet question(words: string, action: string)}
  <ConfirmCard label="Confirm service action">
    <span>{words}</span>
    <button class="btn p" type="button"><Icon icon={CheckIcon} size={14} state="strong" />{action}</button>
    <button class="btn" type="button">Cancel</button>
  </ConfirmCard>
{/snippet}

<Story name="Default" asChild>
  <div class="col">{@render question("Restart Local search on laptop? Search that uses this database may be unavailable while it is stopped or restarting.", "Confirm restart")}</div>
</Story>

<Story name="Lengths" asChild>
  <Cases width={640}>
    <Case label="a start" note="nothing is lost, so there is nothing to warn about">{@render question("Start Index watcher on laptop?", "Confirm start")}</Case>
    <Case label="a stop">{@render question("Stop Index watcher on laptop? New chats may stop appearing until the watcher is running again. Existing indexed history is retained.", "Confirm stop")}</Case>
    <Case label="narrow" note="the buttons drop under the words"><div class="narrow">{@render question("Restart Mash MCP on laptop? Clients using this service may lose access while it is stopped or restarting.", "Confirm restart")}</div></Case>
  </Cases>
</Story>

<style>
  .col {
    max-width: 640px;
  }
  .narrow {
    width: 320px;
  }
</style>
