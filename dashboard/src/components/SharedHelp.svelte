<script lang="ts">
import KeyRound from "@lucide/svelte/icons/key-round";
import ListChecks from "@lucide/svelte/icons/list-checks";
import Lock from "@lucide/svelte/icons/lock";
import SquareTerminal from "@lucide/svelte/icons/square-terminal";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import { joinRefusals } from "../lib/shared";
import CommandCopy from "./CommandCopy.svelte";

// The notes beside the shared server's main card: what each step leaves on
// this machine, and the terminal commands that do the same. Every command and
// fact here is the one docs/shared-index.md documents; none is a guess about
// this server.
let { mode, node = "" }: { mode: "review" | "done" | "join" | "admin"; node?: string } = $props();
const here = $derived(node || "this machine");
</script>

{#if mode === "review"}
  <section class="card sh" aria-label="What stays private">
    <h3><Lock size={15} aria-hidden="true" />What stays private</h3>
    <p>Chats you leave out are still indexed and searchable on {here}, and so are any memories drawn from them; neither is sent.</p>
    <p>From a terminal, list an agent under <code class="i">[shared]</code> in the configuration to keep it off the server, for example <code class="i">unshared = ["cursor"]</code>.</p>
  </section>
{:else if mode === "done"}
  <section class="card sh" aria-label="Next steps">
    <h3><ListChecks size={15} aria-hidden="true" />Next steps</h3>
    <ol class="sh-steps">
      <li><strong>Search every machine</strong><p>Search on any joined computer covers every machine in the group. The machine filter narrows it to the ones you choose.</p></li>
      <li><strong>Check what the server received</strong><CommandCopy text="mash status --shared" /></li>
      <li><strong>Keep publishing new chats</strong><p>Run automatic indexing on each computer, so new chats reach the server as Mash indexes them.</p></li>
    </ol>
  </section>
{:else if mode === "join"}
  <section class="card sh" aria-label="Join from a terminal">
    <h3><SquareTerminal size={15} aria-hidden="true" />Join from a terminal</h3>
    <p>On this computer, with the full join value. These options skip indexing and leave agent settings alone.</p>
    <CommandCopy text="mash init --join 'JOIN_STRING_FROM_SERVER' --no-index --no-agents" />
  </section>
  <section class="card sh" aria-label="Connect by hand">
    <h3><KeyRound size={15} aria-hidden="true" />Connect by hand</h3>
    <p>No join string? Ask the server’s operator for the server URL, the group ID, a name for this computer (its node ID) and its own token.</p>
    <CommandCopy text="mash init --no-index --no-agents" />
    <p>Choose Connect to an existing shared server, then enter the server URL, group ID, node ID and the name of the variable that holds the token.</p>
  </section>
  <section class="card sh" aria-label="When the server says no">
    <h3><TriangleAlert size={15} aria-hidden="true" />When the server says no</h3>
    <p>Mash shows why, and saves nothing.</p>
    <dl class="sh-why">
      <dt>Expired</dt><dd>{joinRefusals.expired}</dd>
      <dt>Already used</dt><dd>{joinRefusals.used}</dd>
      <dt>Replaced</dt><dd>{joinRefusals.revoked}</dd>
    </dl>
  </section>
{:else}
  <section class="card sh" aria-label="On a machine without an admin key">
    <h3><KeyRound size={15} aria-hidden="true" />On a machine without an admin key</h3>
    <p>Every joined machine sees the server’s machines with its own credential, which can never change anything. To manage from it, choose Manage this server there and paste a key.</p>
  </section>
  <section class="card sh" aria-label="On the server machine">
    <h3><SquareTerminal size={15} aria-hidden="true" />On the server machine</h3>
    <p>Make a key for another computer. It is printed once; the server keeps only its hash.</p>
    <CommandCopy text="mash-server --server-config /absolute/path/to/server.toml --add-admin NAME" />
    <p>Stop accepting a key, for example for a lost laptop.</p>
    <CommandCopy text="mash-server --server-config /absolute/path/to/server.toml --revoke-admin NAME" />
    <p>A server with several groups adds machines there, not in the dashboard.</p>
    <CommandCopy text="mash-server --add-node GROUP/NODE" />
  </section>
{/if}

<style>
  .sh {
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-size: 13.5px;
    line-height: 1.5;
  }
  .sh > :global(*) {
    margin: 0;
  }
  h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-strong);
  }
  .sh-steps {
    display: grid;
    gap: 14px;
    padding: 0;
    list-style: none;
    counter-reset: step;
  }
  .sh-steps li {
    display: grid;
    grid-template-columns: 22px minmax(0, 1fr);
    gap: 6px 12px;
    counter-increment: step;
  }
  .sh-steps li::before {
    content: counter(step);
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border: var(--border-outline);
    border-radius: var(--radius-round);
    font-size: 11.5px;
    color: var(--text-muted);
  }
  .sh-steps li > :global(*) {
    grid-column: 2;
    margin: 0;
  }
  .sh-steps li > strong {
    grid-row: 1;
    align-self: center;
    color: var(--text-strong);
  }
  .sh-why {
    display: grid;
    margin: 0;
  }
  .sh-why dt {
    padding-top: 10px;
    border-top: var(--border-divider);
    font-size: 12.5px;
    color: var(--text-muted);
  }
  .sh-why dd {
    margin: 2px 0 10px;
    color: var(--status-warn-text);
  }
</style>
