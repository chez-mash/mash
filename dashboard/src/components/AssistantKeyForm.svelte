<script lang="ts">
import Route from "@lucide/svelte/icons/route";
import Save from "@lucide/svelte/icons/save";
import Server from "@lucide/svelte/icons/server";
import { useQueryClient } from "@tanstack/svelte-query";
import { agents } from "../lib/agents";
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
let model = $state("");
let saving = $state(false);
let failure = $state("");
const chosen = $derived(assistantServices.find((option) => option.id === service));
// A service without a model of our choosing asks for one, as does another service.
const asksModel = $derived(!chosen?.model);
const ready = $derived(
  !!apiKey.trim() &&
    (!withService || ((!!chosen || !!otherUrl.trim()) && (!asksModel || !!model.trim()))),
);
// Each service's mark: the providers' own logos where Mash has them.
const marks: Record<string, string> = {
  anthropic: agents["claude-code"].logo,
  openai: agents.codex.logo,
};

async function save(): Promise<void> {
  if (saving || !ready) return;
  saving = true;
  failure = "";
  try {
    const target = {
      baseUrl: chosen ? chosen.baseUrl : otherUrl,
      model: asksModel ? model : (chosen?.model ?? model),
    };
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
    <fieldset class="services">
      <legend class="set-l">model service<small>any service that speaks the OpenAI chat format works</small></legend>
      <div class="service-tiles">
        {#each [...assistantServices, { id: "other", name: "Another service" }] as option (option.id)}
          <label class="service-tile" class:on={service === option.id}>
            <input type="radio" name="{id}-service" value={option.id} bind:group={service} />
            <span class="service-mark {option.id}" aria-hidden="true">
              {#if marks[option.id]}{@html marks[option.id]}{:else if option.id === "openrouter"}<Route size={18} />{:else}<Server size={18} />{/if}
            </span>
            <span>{option.name}</span>
          </label>
        {/each}
      </div>
    </fieldset>
    {#if !chosen}
      <label class="set-l" for="{id}-url">API address<small>its base, such as https://api.example.com/v1</small></label>
      <input id="{id}-url" class="field" type="url" autocomplete="off" spellcheck="false" bind:value={otherUrl} />
    {/if}
    {#if asksModel}
      <label class="set-l" for="{id}-model">model<small>as {chosen ? chosen.name : "that service"} names it; you can change it later in settings</small></label>
      <input id="{id}-model" class="field" autocomplete="off" spellcheck="false" bind:value={model} />
    {/if}
  {/if}
  <label class="set-l" for="{id}-key">API key<small>saved on the computer running the dashboard, readable only by you{#if withService && chosen}; <a class="a" href={chosen.keys} target="_blank" rel="noreferrer">get an API key from {chosen.name}</a>{/if}</small></label>
  <input id="{id}-key" class="field" type="password" autocomplete="off" spellcheck="false" bind:value={apiKey} />
  {#if failure}<p class="meta warn" role="alert">{failure}</p>{/if}
  <span class="keyacts">
    <button class="btn p" type="submit" disabled={saving || !ready}><Save size={14} aria-hidden="true" />{saving ? "Saving…" : withService ? "Turn on Mashy" : "Save key"}</button>
    {#if oncancel}<button class="btn" type="button" onclick={() => { apiKey = ""; failure = ""; oncancel(); }}>Cancel</button>{/if}
  </span>
</form>

<style>
  /* The chosen service, as in the wireframes: a visible radio, a tinted tile
     with a firmer ring, and each provider's mark on its own colour. */
  .service-tile {
    border-color: var(--line-outline);
    font-weight: var(--weight-label);
    font-size: 13px;
  }
  .service-tile.on {
    border-color: transparent;
    background: var(--selected-surface);
    color: var(--text-strong);
    box-shadow: 0 0 0 1.5px var(--selected-ring);
    font-weight: var(--weight-strong);
  }
  .service-tile input {
    position: static;
    flex: none;
    margin: 0;
    opacity: 1;
    accent-color: var(--text-strong);
  }
  .service-mark {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-4);
    background: var(--surface-muted);
    color: var(--text-body);
  }
  .service-mark :global(svg) {
    width: 14px;
    height: 14px;
  }
  .service-mark.anthropic {
    background: var(--agent-claude-code);
    color: var(--agent-on);
  }
  .service-mark.openai {
    background: var(--agent-codex);
    color: var(--agent-on-neutral);
  }
</style>
