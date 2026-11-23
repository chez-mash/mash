<script lang="ts">
import { useQueryClient } from "@tanstack/svelte-query";
import { assistantOptions } from "../lib/assistant";
import { assistantServices, type Settings, settingsOptions, setUpAssistant } from "../lib/settings";

// Sets Mashy up from a pasted API key: choose a model service (or keep the
// configured one) and save the key on the computer running the dashboard,
// where it is readable only by its user and never shown again. Settings and
// Mashy's own panel both use it, so the two always ask the same way.
let {
  settings,
  withService,
  id,
  oncancel,
  onsaved,
}: {
  settings: Settings;
  /** Ask for the service as well as the key; without it only the key changes. */
  withService: boolean;
  /** Distinguishes this form's fields when another copy is on the page. */
  id: string;
  oncancel?: () => void;
  onsaved?: () => void;
} = $props();

const queryClient = useQueryClient();
let service = $state<string>(assistantServices[0].id);
let apiKey = $state("");
let otherUrl = $state("");
let otherModel = $state("");
let saving = $state(false);
let failure = $state("");
const chosen = $derived(assistantServices.find((option) => option.id === service));
const ready = $derived(
  !!apiKey.trim() && (!withService || !!chosen || (!!otherUrl.trim() && !!otherModel.trim())),
);

async function save(): Promise<void> {
  if (saving || !ready) return;
  saving = true;
  failure = "";
  try {
    const target = chosen ?? { baseUrl: otherUrl, model: otherModel };
    const next = await setUpAssistant(settings, apiKey, withService ? target : undefined);
    apiKey = "";
    queryClient.setQueryData(settingsOptions.queryKey, next);
    await queryClient.invalidateQueries({ queryKey: assistantOptions.queryKey });
    onsaved?.();
  } catch (error) {
    failure = error instanceof Error ? error.message : "Mashy was not set up.";
  } finally {
    saving = false;
  }
}
</script>

<form class="keyform" onsubmit={(event) => { event.preventDefault(); void save(); }}>
  {#if withService}
    <label class="set-l" for="{id}-service">model service<small>any service that speaks the OpenAI chat format works</small></label>
    <select id="{id}-service" class="field" bind:value={service}>
      {#each assistantServices as option (option.id)}<option value={option.id}>{option.name}</option>{/each}
      <option value="other">Another service</option>
    </select>
    {#if !chosen}
      <label class="set-l" for="{id}-url">API address<small>its base, such as https://api.example.com/v1</small></label>
      <input id="{id}-url" class="field" type="url" autocomplete="off" spellcheck="false" bind:value={otherUrl} />
      <label class="set-l" for="{id}-model">model<small>as that service names it</small></label>
      <input id="{id}-model" class="field" autocomplete="off" spellcheck="false" bind:value={otherModel} />
    {/if}
  {/if}
  <label class="set-l" for="{id}-key">API key<small>saved on the computer running the dashboard, readable only by you{#if withService && chosen}; <a class="a" href={chosen.keys} target="_blank" rel="noreferrer">get an API key from {chosen.name}</a>{/if}</small></label>
  <input id="{id}-key" class="field" type="password" autocomplete="off" spellcheck="false" bind:value={apiKey} />
  {#if failure}<p class="meta warn" role="alert">{failure}</p>{/if}
  <span class="keyacts">
    <button class="btn p" type="submit" disabled={saving || !ready}>{saving ? "Saving…" : withService ? "Turn on Mashy" : "Save key"}</button>
    {#if oncancel}<button class="btn" type="button" onclick={() => { apiKey = ""; failure = ""; oncancel(); }}>Cancel</button>{/if}
  </span>
</form>
