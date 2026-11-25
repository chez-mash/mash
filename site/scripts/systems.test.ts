import { describe, expect, it } from "vitest";
import { systemsOf } from "./systems.ts";

describe("a code window's systems", () => {
  it("come from the systems its title names", () => {
    expect(systemsOf("macOS (Apple silicon) or Linux (x86-64 or ARM64)", "sh")).toEqual([
      "macos",
      "linux",
    ]);
    expect(systemsOf("Windows x64 or ARM64 (PowerShell)", "powershell")).toEqual(["windows"]);
    expect(systemsOf("Linux", "sh")).toEqual(["linux"]);
  });

  it("include Windows for a titled PowerShell window", () => {
    expect(systemsOf("Install", "powershell")).toEqual(["windows"]);
  });

  it("are none for a window that names no system", () => {
    expect(systemsOf("", "powershell")).toEqual([]);
    expect(systemsOf("config.toml", "toml")).toEqual([]);
    // A word that merely contains a system's letters is not one.
    expect(systemsOf("Machines", "sh")).toEqual([]);
  });
});
