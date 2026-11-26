<script lang="ts">
// A sheet of one component's states: each `Case` inside it is a labelled
// cell. `row` lays small pieces side by side; the default stacks them, each
// at the width it would have in a column of the dashboard.
import type { Snippet } from "svelte";

let {
  children,
  row = false,
  width,
}: {
  children: Snippet;
  row?: boolean;
  /** The widest a stacked case is drawn, in px. */
  width?: number | undefined;
} = $props();
</script>

<div class="cases" class:row style:max-width={width === undefined ? undefined : `${width}px`}>{@render children()}</div>

<style>
  .cases {
    display: flex;
    flex-direction: column;
    gap: 22px;
  }
  .cases.row {
    flex-flow: row wrap;
    align-items: flex-start;
    gap: 20px 28px;
  }
</style>
