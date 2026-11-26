import BookmarkSimpleIcon from "phosphor-svelte/lib/BookmarkSimpleIcon";
import ChatIcon from "phosphor-svelte/lib/ChatIcon";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import GitBranchIcon from "phosphor-svelte/lib/GitBranchIcon";
import NoteIcon from "phosphor-svelte/lib/NoteIcon";
import SlidersHorizontalIcon from "phosphor-svelte/lib/SlidersHorizontalIcon";
import UserIcon from "phosphor-svelte/lib/UserIcon";

/** The icon for each kind of memory, the same wherever a kind is shown. */
export const kindIcons = {
  decision: BookmarkSimpleIcon,
  preference: SlidersHorizontalIcon,
  note: NoteIcon,
} as const;

/** The icon for what a memory applies to. */
export const scopeIcons = {
  personal: UserIcon,
  repository: GitBranchIcon,
  local_project: FolderIcon,
  chat: ChatIcon,
} as const;
