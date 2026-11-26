// The glyphs that stand for a section of the dashboard. Each is drawn with a
// `ph-NAME` class so a heading, a chip or a folded step can give it its
// section's colour (see "An icon keeps its section's colour" in terminal.css).
import BrainIcon from "phosphor-svelte/lib/BrainIcon";
import ChatsIcon from "phosphor-svelte/lib/ChatsIcon";
import ClockIcon from "phosphor-svelte/lib/ClockIcon";
import FileTextIcon from "phosphor-svelte/lib/FileTextIcon";
import FolderIcon from "phosphor-svelte/lib/FolderIcon";
import HardDrivesIcon from "phosphor-svelte/lib/HardDrivesIcon";
import InfoIcon from "phosphor-svelte/lib/InfoIcon";
import LaptopIcon from "phosphor-svelte/lib/LaptopIcon";
import LightbulbIcon from "phosphor-svelte/lib/LightbulbIcon";
import LinkSimpleHorizontalIcon from "phosphor-svelte/lib/LinkSimpleHorizontalIcon";
import RobotIcon from "phosphor-svelte/lib/RobotIcon";
import TerminalIcon from "phosphor-svelte/lib/TerminalIcon";
import WarningIcon from "phosphor-svelte/lib/WarningIcon";

const names = new Map<unknown, string>([
  [BrainIcon, "brain"],
  [ChatsIcon, "chats"],
  [ClockIcon, "clock"],
  [FileTextIcon, "file-text"],
  [FolderIcon, "folder"],
  [HardDrivesIcon, "hard-drives"],
  [InfoIcon, "info"],
  [LaptopIcon, "laptop"],
  [LightbulbIcon, "lightbulb"],
  [LinkSimpleHorizontalIcon, "link"],
  [RobotIcon, "robot"],
  [TerminalIcon, "terminal"],
  [WarningIcon, "warning"],
]);

/** The class name of a section's glyph, or undefined for any other icon. */
export function glyphName(glyph: unknown): string | undefined {
  return names.get(glyph);
}
