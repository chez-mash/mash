<script lang="ts">
import HouseIcon from "phosphor-svelte/lib/HouseIcon";
import Icon from "$lib/components/Icon.svelte";
import EmptyState from "./EmptyState.svelte";
</script>

<EmptyState card title="Page not found" heading="h1">
  This link does not match a page in this dashboard.
  {#snippet action()}<a class="btn p" href="#/"><Icon icon={HouseIcon} size={15} state="strong" />Go to overview</a>{/snippet}
</EmptyState>
