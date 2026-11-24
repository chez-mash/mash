// Which operating system a machine runs, and its mark.
//
// This dashboard's own service check names the system of each machine it
// checks; otherwise the shared server says what the machine named when it
// last checked in. A machine neither has seen keeps the plain laptop icon.
import { type Os, reported, type ServerReport } from "./serverStatus";
import type { Machine } from "./services.svelte";

const names: Record<Os, string> = { macos: "macOS", linux: "Linux", windows: "Windows" };
const files: Record<Os, string> = {
  macos: "apple.svg",
  linux: "linux.svg",
  windows: "windows.svg",
};

// Inlined, like the agent marks, so a mark takes the colour around it.
const svgs = import.meta.glob<string>("../../public/os/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

function isOs(value: string): value is Os {
  return value in names;
}

export function machineOs(
  name: string,
  fleet: readonly Machine[] | undefined,
  report: ServerReport,
): Os | null {
  const checked = fleet?.find((machine) => machine.id === name)?.platform;
  if (checked && isOs(checked)) return checked;
  return reported(report, name)?.os ?? null;
}

export const osName = (os: Os): string => names[os];

/** The system's mark as SVG markup filled with the current colour. */
export function osLogo(os: Os): string {
  const svg = Object.entries(svgs).find(([path]) => path.endsWith(`/os/${files[os]}`))?.[1] ?? "";
  return svg
    .replace(/<title>.*?<\/title>/s, "")
    .replace(/\s(width|height|style|fill|role)="[^"]*"/g, "")
    .replace("<svg", '<svg fill="currentColor" aria-hidden="true" focusable="false"');
}
