<script lang="ts">
import Folder from "@lucide/svelte/icons/folder";
import Laptop from "@lucide/svelte/icons/laptop";
import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
import { tick } from "svelte";
import AgentMark from "./components/AgentMark.svelte";
import BrowseDetail from "./components/BrowseDetail.svelte";
import BrowseTiles from "./components/BrowseTiles.svelte";
import Console from "./components/Console.svelte";
import Fleet from "./components/Fleet.svelte";
import History from "./components/History.svelte";
import MachineExtras from "./components/MachineExtras.svelte";
import Mashy from "./components/Mashy.svelte";
import Missing from "./components/Missing.svelte";
import Overview from "./components/Overview.svelte";
import Projects from "./components/Projects.svelte";
import QuickSearch from "./components/QuickSearch.svelte";
import Rail from "./components/Rail.svelte";
import RecordsView from "./components/RecordsView.svelte";
import RecordView from "./components/RecordView.svelte";
import Search from "./components/Search.svelte";
import ServicesView from "./components/ServicesView.svelte";
import Settings from "./components/Settings.svelte";
import Setup from "./components/Setup.svelte";
import SharedServer from "./components/SharedServer.svelte";
import SignIn from "./components/SignIn.svelte";
import TopBar from "./components/TopBar.svelte";
import TranscriptView from "./components/TranscriptView.svelte";
import Transfer from "./components/Transfer.svelte";
import { agents } from "./lib/agents";
import { isAgent } from "./lib/api";
import { projectLabel } from "./lib/browse";
import { connect, onChange, throttle } from "./lib/live";
import { catalogOptions, stableRead } from "./lib/queries";
import { router } from "./lib/route.svelte";
import { check, gate, open } from "./lib/session.svelte";
import { firstRunOptions, opensOnSetup } from "./lib/setup";

const client = new QueryClient({ defaultOptions: { queries: stableRead } });
// Nothing is read until the dashboard has said whether it wants a sign-in.
void check();
function signedIn(): void {
  // Answers refused before the sign-in must not be shown as failures after it.
  client.clear();
  open();
}
// One stream per tab says when the index changed. Chat lists follow it,
// at most once every few seconds while indexing is busy.
$effect(() => {
  if (gate.value !== "open") return;
  const refresh = throttle(
    () => void client.invalidateQueries({ queryKey: catalogOptions.queryKey }),
    5000,
  );
  // Lists of memories follow too. A memory search keeps its results, like
  // any submitted search, and an open record is reread only on request.
  const memories = throttle(
    () =>
      void client.invalidateQueries({
        predicate: ({ queryKey: [kind, params] }) =>
          kind === "chat-memory" ||
          (kind === "records" &&
            typeof params === "string" &&
            !new URLSearchParams(params).has("q")),
      }),
    5000,
  );
  const off = onChange((change) => {
    refresh();
    if (change.all || change.records) memories();
  });
  // A stream refused because the session ended should bring up the sign-in.
  const stop = connect(() => void check());
  return () => {
    stop();
    off();
    refresh.cancel();
    memories.cancel();
  };
});
const route = $derived(router.current);
// A machine that has not been set up opens on setup, once per visit.
$effect(() => {
  if (gate.value !== "open") return;
  let current = true;
  client
    .fetchQuery(firstRunOptions)
    .then((setup) => {
      if (current && opensOnSetup(setup, router.current.page)) location.hash = "#/setup";
    })
    .catch(() => {
      // A dashboard that cannot describe setup simply opens as usual.
    });
  return () => {
    current = false;
  };
});
// Keep the submitted search mounted while a chat is open, so going back
// returns to the same results.
let searchParams = $state<string | null>(null);
$effect(() => {
  if (route.page === "search") searchParams = route.params;
});
// The quick panel draws in the system's look; the rest in the theme's.
$effect(() => {
  if (route.page === "quick") document.documentElement.dataset.surface = "native";
  else delete document.documentElement.dataset.surface;
});
// A detail page reads best at a narrower measure than a list or a table.
const narrow = $derived(
  route.page === "record" ||
    (route.page === "fleet" && !!route.action) ||
    (route.page === "services" && new URLSearchParams(route.params).has("service")),
);
let railOpen = $state(false);
$effect(() => {
  void route;
  railOpen = false;
});
// On a narrow screen the rail is a drawer over the page. While it is open it
// holds the keyboard: focus moves into it, the page behind is inert, and Tab
// cycles through the drawer. Closing it returns focus to the menu button,
// unless the reader has already moved focus somewhere of their own.
const drawerQuery =
  typeof window === "undefined" ? undefined : window.matchMedia("(max-width: 800px)");
$effect(() => {
  if (!drawerQuery) return;
  const widened = (): void => {
    if (!drawerQuery.matches) railOpen = false;
  };
  drawerQuery.addEventListener("change", widened);
  return () => drawerQuery.removeEventListener("change", widened);
});
let drawerWasOpen = false;
$effect(() => {
  if (railOpen) {
    drawerWasOpen = true;
    void tick().then(() => document.getElementById("xdrawer")?.focus());
    return;
  }
  if (!drawerWasOpen) return;
  drawerWasOpen = false;
  const active = document.activeElement;
  if (!active || active === document.body || document.getElementById("rail")?.contains(active))
    document.getElementById("burger")?.focus();
});
function drawerStops(): HTMLElement[] {
  const rail = document.getElementById("rail");
  if (!rail) return [];
  return [
    ...rail.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input, select, textarea, [tabindex]:not([tabindex="-1"])',
    ),
  ].filter((element) => element.tabIndex >= 0 && element.getClientRects().length > 0);
}
function keepInDrawer(event: KeyboardEvent): void {
  const stops = drawerStops();
  const first = stops[0];
  const last = stops.at(-1);
  if (!first || !last) return;
  const active = document.activeElement;
  const inside = active instanceof HTMLElement && stops.includes(active);
  if (!inside) first.focus();
  else if (event.shiftKey && active === first) last.focus();
  else if (!event.shiftKey && active === last) first.focus();
  else return;
  event.preventDefault();
}
// The skip link lands on the page's content, past the rail and the search bar.
function skip(event: MouseEvent): void {
  event.preventDefault();
  document.getElementById("main")?.focus();
}
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape" && railOpen) {
      railOpen = false;
      document.getElementById("burger")?.focus();
    } else if (event.key === "Tab" && railOpen) keepInDrawer(event);
  }}
/>
{#if gate.value === "locked"}
  <SignIn done={signedIn} />
{:else if gate.value === "open"}
<QueryClientProvider {client}>
  {#if route.page === "setup"}
    <!-- Setup is a screen of its own: before it, the rail and search lead nowhere. -->
    <Setup />
  {:else if route.page === "quick"}
    <!-- The desktop app's menu bar panel: search alone, in the system's look. -->
    <QuickSearch />
  {:else}
  <a class="skip" href="#main" onclick={skip}>Skip to content</a>
  <Rail open={railOpen} close={() => (railOpen = false)} />
  <div id="col" inert={railOpen}>
    <TopBar {railOpen} toggleRail={() => (railOpen = !railOpen)} />
    <main id="main" class:narrow={narrow} tabindex="-1">
      {#if searchParams !== null}
        <div class="stack" hidden={route.page !== "search"}>
          {#key searchParams}<Search params={searchParams} />{/key}
        </div>
      {/if}
      {#if route.page === "overview"}
        <Overview />
      {:else if route.page === "search"}
        <!-- Rendered above so it survives opening a chat. -->
      {:else if route.page === "history"}
        <History />
      {:else if route.page === "export"}
        <Transfer params={route.params} />
      {:else if route.page === "projects"}
        <Projects />
      {:else if route.page === "project"}
        {#key route.name}
          <BrowseDetail field="project" value={route.name} title={projectLabel(route.name)} back={{ href: "#/projects", label: "projects" }}>
            {#snippet icon()}<span class="ico lg projects"><Folder size={20} aria-hidden="true" /></span>{/snippet}
          </BrowseDetail>
        {/key}
      {:else if route.page === "agents"}
        <BrowseTiles field="agent" />
      {:else if route.page === "agent"}
        {@const agent = route.agent}
        {#if isAgent(agent)}
          {#key agent}
            <BrowseDetail field="agent" value={agent} title={agents[agent].name} back={{ href: "#/agents", label: "agents" }}>
              {#snippet icon()}<AgentMark {agent} size="lg" />{/snippet}
            </BrowseDetail>
          {/key}
        {:else}
          <Missing />
        {/if}
      {:else if route.page === "machines"}
        <BrowseTiles field="machine" />
        <SharedServer joinable />
      {:else if route.page === "machine"}
        {@const id = route.id}
        {#key id}
          <BrowseDetail field="machine" value={id} title={id} back={{ href: "#/machines", label: "machines" }}>
            {#snippet icon()}<span class="ico lg machines"><Laptop size={20} aria-hidden="true" /></span>{/snippet}
            {#snippet extra()}<MachineExtras machineId={id} />{/snippet}
          </BrowseDetail>
        {/key}
      {:else if route.page === "services"}
        <ServicesView params={route.params} />
      {:else if route.page === "fleet"}
        <Fleet action={route.action} />
      {:else if route.page === "console"}
        <Console />
      {:else if route.page === "settings"}
        <Settings />
      {:else if route.page === "records"}
        {#key route.params}<RecordsView params={route.params} />{/key}
      {:else if route.page === "record"}
        {#key JSON.stringify([route.id, route.node])}<RecordView id={route.id} node={route.node} back={route.back} />{/key}
      {:else if route.page === "chat"}
        <TranscriptView id={route.id} node={route.node} messageId={route.message} shared={route.shared} />
      {:else}
        <Missing />
      {/if}
    </main>
  </div>
  <!-- The drawer's own button and Escape close it for keyboards and screen readers. -->
  {#if railOpen}<div id="scrim" aria-hidden="true" onclick={() => (railOpen = false)}></div>{/if}
  <div class="mashy-host" inert={railOpen}><Mashy /></div>
  {/if}
</QueryClientProvider>
{/if}
