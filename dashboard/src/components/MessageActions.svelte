<script lang="ts">
import Copy from "@lucide/svelte/icons/copy";
import Link2 from "@lucide/svelte/icons/link-2";
import { saidText } from "../lib/message";
import { shareLink } from "../lib/shareLink";
import type { Message } from "../lib/types";

let { message, href }: { message: Message; href: string } = $props();
let notice = $state("");

async function copy(kind: "message" | "link"): Promise<void> {
  notice = "";
  try {
    const value =
      kind === "link"
        ? shareLink(href, `${location.origin}${location.pathname}`)
        : saidText(message);
    await navigator.clipboard.writeText(value);
    notice = kind === "link" ? "Copied message link." : "Copied message.";
  } catch {
    notice =
      kind === "link"
        ? "Couldn’t copy the message link."
        : "Copy failed. Select the message text to copy it manually.";
  }
}
</script>

<div class="cp-actions">
  <span class="meta" role="status">{notice}</span>
  <div class="cp-buttons">
    <button class="cp" type="button" aria-label="Copy message" title="Copy message" onclick={() => copy("message")}>
      <Copy size={14} aria-hidden="true" />
    </button>
    <button class="cp" type="button" aria-label="Copy link to message" title="Copy link to message" onclick={() => copy("link")}>
      <Link2 size={14} aria-hidden="true" />
    </button>
  </div>
</div>
