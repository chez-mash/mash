/**
 * The operating systems a code window is for, read from its title and
 * language, so the window's bar can show their marks beside the title:
 * "macOS (Apple silicon) or Linux" shows the Apple and Linux marks, and a
 * PowerShell window the Windows one. A window that names no system shows none.
 */
export type System = "macos" | "linux" | "windows";

export function systemsOf(title: string, language: string): System[] {
  const systems: System[] = [];
  if (/\bmac(OS)?\b/i.test(title)) systems.push("macos");
  if (/\bLinux\b/i.test(title)) systems.push("linux");
  if (/\bWindows\b|\bPowerShell\b/i.test(title) || (title !== "" && language === "powershell"))
    systems.push("windows");
  return systems;
}

interface Element {
  properties: Record<string, unknown>;
}

interface BlockContext {
  codeBlock: { language: string; props: { title?: string | undefined } };
  renderData: { blockAst: Element };
}

/** An Expressive Code plugin that marks each window with its systems. */
export function systemMarks() {
  return {
    name: "mash-system-marks",
    hooks: {
      postprocessRenderedBlock({ codeBlock, renderData }: BlockContext) {
        const systems = systemsOf(codeBlock.props.title ?? "", codeBlock.language);
        if (systems.length > 0) renderData.blockAst.properties.dataSystems = systems.join(" ");
      },
    },
  };
}
