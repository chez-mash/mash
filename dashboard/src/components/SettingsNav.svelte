<script lang="ts">
import { settingsHref } from "../lib/route.svelte";

// The settings sections as a list beside them. The section nearest the top
// of the screen is marked, so the reader can tell where they are on a long
// page; choosing one moves to it through the route, as Mashy's link does.
let { sections }: { sections: readonly { readonly id: string; readonly label: string }[] } =
  $props();

let current = $state("");
// Before any section has passed the top, the first one is where the reader is.
const marked = $derived(current || sections[0]?.id);
$effect(() => {
  const shown = new Set<string>();
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const id = entry.target.id.replace(/^set-/, "");
        if (entry.isIntersecting) shown.add(id);
        else shown.delete(id);
      }
      current = sections.find((section) => shown.has(section.id))?.id ?? current;
    },
    // Only the band at the top of the screen counts as where the reader is.
    { rootMargin: "-80px 0px -60% 0px" },
  );
  for (const section of sections) {
    const element = document.getElementById(`set-${section.id}`);
    if (element) observer.observe(element);
  }
  return () => observer.disconnect();
});
</script>

<nav class="set-nav" aria-label="Settings sections">
  {#each sections as section (section.id)}
    <a href={settingsHref(section.id)} aria-current={marked === section.id ? "location" : undefined}>{section.label}</a>
  {/each}
</nav>
