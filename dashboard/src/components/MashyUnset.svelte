<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import { asset } from "../lib/asset";
import { settingsHref } from "../lib/route.svelte";
import { settingsOptions } from "../lib/settings";
import AssistantKeyForm from "./AssistantKeyForm.svelte";

// What Mashy shows instead of questions to try when it has no model to ask.
// Where this dashboard may change settings, it asks for the API key right
// here. A dashboard reached from another machine does not change settings,
// so there it says where that is done instead.
let { leave }: { leave: () => void } = $props();
const settings = createQuery(() => settingsOptions);
</script>

<div class="mashy-hello mashy-unset">
  <img src={asset("mashy.png")} alt="" />
  <strong>Mashy isn’t set up yet</strong>
  <p>Mashy answers questions about your past chats using a model service you choose, with your own API key.</p>
  {#if settings.data && !settings.data.editable}
    <p>This dashboard can be reached from other machines, so it doesn’t change settings. Set Mashy up from the dashboard on the computer that runs Mash, or in that computer’s configuration file.</p>
    <a class="a" href="https://getmash.io/configuration/#the-dashboard-assistant" target="_blank" rel="noreferrer">How to set up Mashy</a>
  {:else if settings.data}
    <p>Choose a model service and paste its API key to turn Mashy on.</p>
    <!-- A service already configured keeps its address and model: only the key is asked for. -->
    <AssistantKeyForm settings={settings.data} withService={!settings.data.assistant} id="mashy-panel" />
    <a class="a" href={settingsHref("assistant")} onclick={leave}>More Mashy settings</a>
  {:else if settings.error}
    <a class="btn p" href={settingsHref("assistant")} onclick={leave}>Set up Mashy in Settings</a>
  {/if}
</div>
