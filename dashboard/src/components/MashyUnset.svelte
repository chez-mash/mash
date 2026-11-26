<script lang="ts">
import { createQuery } from "@tanstack/svelte-query";
import BookOpenIcon from "phosphor-svelte/lib/BookOpenIcon";
import GearSixIcon from "phosphor-svelte/lib/GearSixIcon";
import Icon from "$lib/components/Icon.svelte";
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
    <!-- Said before the key is given, in the words of the Mashy guide. -->
    <aside class="mashy-sends">
      <strong>What Mashy sends</strong>
      <p>Your question, recent turns with Mashy, any passage you pin, and the chat text and memories Mashy looks up go to the model service you choose. By default Mash masks recognizable credentials in what it looks up. Nothing is sent until you ask.</p>
      <a class="a" href="https://getmash.io/privacy/#when-history-can-leave-your-machine" target="_blank" rel="noreferrer"><Icon icon={BookOpenIcon} size={13} />When history can leave your machine</a>
    </aside>
  {:else if settings.error}
    <a class="btn p" href={settingsHref("assistant")} onclick={leave}><Icon icon={GearSixIcon} size={14} state="strong" />Set up Mashy in Settings</a>
  {/if}
</div>
