import Bookmark from "@lucide/svelte/icons/bookmark";
import Folder from "@lucide/svelte/icons/folder";
import FolderGit2 from "@lucide/svelte/icons/folder-git-2";
import MessageSquare from "@lucide/svelte/icons/message-square";
import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
import StickyNote from "@lucide/svelte/icons/sticky-note";
import User from "@lucide/svelte/icons/user";

/** The icon for each kind of memory, the same wherever a kind is shown. */
export const kindIcons = {
  decision: Bookmark,
  preference: SlidersHorizontal,
  note: StickyNote,
} as const;

/** The icon for what a memory applies to. */
export const scopeIcons = {
  personal: User,
  repository: FolderGit2,
  local_project: Folder,
  chat: MessageSquare,
} as const;
