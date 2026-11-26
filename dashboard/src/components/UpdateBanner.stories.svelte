<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import type { UpdateOffer } from "../lib/appUpdate.svelte";
import Case from "../stories/Case.svelte";
import Cases from "../stories/Cases.svelte";
import UpdateBanner from "./UpdateBanner.svelte";

// The strip across the top of the desktop app's window when a newer Mash
// exists. The versions here are made up.
const { Story } = defineMeta({ title: "Blocks/UpdateBanner", component: UpdateBanner });

const offer = (changes: Partial<UpdateOffer> = {}): UpdateOffer => ({
  current: "1.4.0",
  version: "1.5.0",
  way: "install",
  stage: "available",
  progress: null,
  reason: null,
  command: null,
  downloads: "https://example.invalid/mash/downloads",
  ...changes,
});
const apt = "sudo apt update && sudo apt install --only-upgrade mash-desktop mash-cli";
const nothing = (): void => {};
</script>

{#snippet banner(shown: UpdateOffer)}
  <UpdateBanner offer={shown} oninstall={nothing} onlater={nothing} />
{/snippet}

<Story name="Update and restart" asChild>
  <div class="window">{@render banner(offer())}</div>
</Story>

<Story name="Download only" asChild>
  <div class="window">{@render banner(offer({ way: "download" }))}</div>
</Story>

<Story name="Downloading" asChild>
  <div class="window">{@render banner(offer({ stage: "downloading", progress: 0.42 }))}</div>
</Story>

<Story name="Failed" asChild>
  <div class="window">{@render banner(offer({ stage: "failed", reason: "the download did not pass its signature check (invalid signature)" }))}</div>
</Story>

<Story name="Other states" asChild>
  <Cases width={900}>
    <Case label="installed with apt" note="the command, and a button that copies it">{@render banner(offer({ way: "apt", command: apt }))}</Case>
    <Case label="downloading, size unknown" note="no bar until the download says how large it is">{@render banner(offer({ stage: "downloading" }))}</Case>
    <Case label="installing" note="nothing to choose; the app restarts by itself">{@render banner(offer({ stage: "installing" }))}</Case>
    <Case label="narrow" note="the buttons drop under the words"><div class="narrow">{@render banner(offer())}</div></Case>
    <Case label="narrow, failed"><div class="narrow">{@render banner(offer({ stage: "failed", reason: "the download is gone (No such file or directory)" }))}</div></Case>
  </Cases>
</Story>

<style>
  .window {
    max-width: 900px;
  }
  .narrow {
    width: 360px;
  }
</style>
