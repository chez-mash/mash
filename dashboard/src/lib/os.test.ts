import { describe, expect, it } from "vitest";
import { machineOs, osLogo } from "./os";
import type { ServerReport } from "./serverStatus";
import type { Machine } from "./services.svelte";

const checked = (id: string, platform: string) =>
  ({ id, platform, checked_at: 0, reachable: true, services: [], warnings: [] }) as Machine;
const report = {
  kind: "reported",
  address: "https://mash.example",
  group: "harbor",
  build: "b",
  now: 0,
  restartNeeded: false,
  machines: [
    { id: "studio", os: "macos" },
    { id: "quiet", os: null },
  ],
} as unknown as ServerReport;

describe("a machine's operating system", () => {
  it("is what this dashboard's check found, before what the server heard", () => {
    expect(machineOs("studio", [checked("studio", "linux")], report)).toBe("linux");
  });

  it("is what the machine said when it checked in, without a check", () => {
    expect(machineOs("studio", undefined, report)).toBe("macos");
  });

  it("is unknown when neither says, or the check names no known system", () => {
    expect(machineOs("quiet", [checked("quiet", "unknown")], report)).toBeNull();
    expect(machineOs("stranger", undefined, { kind: "none" })).toBeNull();
  });

  it("draws each system's mark in the colour around it", () => {
    for (const os of ["macos", "linux", "windows"] as const) {
      expect(osLogo(os)).toContain('fill="currentColor"');
      expect(osLogo(os)).not.toContain("<title>");
    }
  });
});
