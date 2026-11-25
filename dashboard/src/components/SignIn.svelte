<script lang="ts">
import KeyRound from "@lucide/svelte/icons/key-round";
import LogIn from "@lucide/svelte/icons/log-in";
import SquareTerminal from "@lucide/svelte/icons/square-terminal";
import { asset } from "../lib/asset";
import { signIn } from "../lib/session.svelte";
import CommandCopy from "./CommandCopy.svelte";

const { done }: { done: () => void } = $props();

let token = $state("");
let busy = $state(false);
let problem = $state<string | undefined>(undefined);

async function submit(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  if (busy || token.trim() === "") return;
  busy = true;
  problem = await signIn(token.trim());
  busy = false;
  if (problem === undefined) {
    token = "";
    done();
  }
}
</script>

<main class="signin">
  <header class="signin-top"><span class="brand"><img src={asset("mash-mark.svg")} alt="" /><span>mash<i class="pat" aria-hidden="true"></i></span></span></header>
  <div class="signin-col">
    <form onsubmit={submit}>
      <span class="signin-key" aria-hidden="true"><KeyRound size={20} /></span>
      <h1 class="h1">Sign in to Mash</h1>
      <p class="signin-why">
        This dashboard can be reached from other devices, so it asks you to sign in. Enter a one-time code from the
        machine that runs it, or its access token.
      </p>
      <label class="signin-l" for="signin-token">Code or access token</label>
      <!-- svelte-ignore a11y_autofocus: the token is the only thing to do on this screen. -->
      <input
        id="signin-token"
        class="field"
        type="text"
        autocomplete="one-time-code"
        autocapitalize="off"
        autocorrect="off"
        spellcheck="false"
        autofocus
        bind:value={token}
        aria-describedby={problem === undefined ? undefined : "signin-problem"}
        aria-invalid={problem !== undefined}
      />
      {#if problem !== undefined}<p id="signin-problem" class="signin-problem" role="alert">{problem}</p>{/if}
      <button class="btn p" type="submit" disabled={busy || token.trim() === ""}><LogIn size={14} aria-hidden="true" />{busy ? "Signing in…" : "Sign in"}</button>
    </form>
    <section class="card signin-help" aria-label="No code yet?">
      <h2><SquareTerminal size={15} aria-hidden="true" />No code yet?</h2>
      <p>On the computer that runs Mash, ask for a new one. It works once, within ten minutes.</p>
      <CommandCopy text="mash dash remote --code" />
      <p>Or scan the QR code that <code class="i">mash dash remote --qr</code> shows there with a phone’s camera.</p>
    </section>
    <p class="signin-note">This browser stays signed in for 30 days.</p>
  </div>
</main>
