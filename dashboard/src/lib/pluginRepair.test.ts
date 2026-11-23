import { describe, expect, it } from "vitest";
import { type RepairDeps, type RepairStage, repairPlugin } from "./pluginRepair";
import type { FleetJob, FleetRequest, Operation } from "./services.svelte";

function job(request: FleetRequest, outcome: FleetJob["outcome"], detail: string | null = null) {
  return {
    request,
    requested_at: 1,
    completed_at: outcome === "running" ? null : 2,
    outcome,
    targets: [{ id: "macbook", state: "running", detail, progress: null }],
  } as FleetJob;
}
function run(id: string, outcome: Operation["outcome"], message = "") {
  return {
    request: {
      machine: "macbook",
      service: "agent-plugins",
      action: "reinstall_plugin",
      request_id: id,
      control_revision: "build",
    },
    requested_at: 1,
    completed_at: null,
    outcome,
    message,
  } as Operation;
}

/** A fake fleet and machine: each read moves a running job or pending run to its end. */
function fake(
  deploy: FleetJob["outcome"],
  reinstall: Operation["outcome"] | Error,
  builds: (string | Error)[] = [],
) {
  const calls: string[] = [];
  let ids = 0;
  let submitted: FleetRequest | null = null;
  const deps: RepairDeps = {
    requestId: () => `id-${++ids}`,
    sleep: async () => {},
    async submitFleetJob(request) {
      calls.push(`deploy ${request.targets.join(",")} ${request.revision}`);
      submitted = request;
      return job(request, "running", "Building");
    },
    async fleetHistory() {
      return submitted ? [job(submitted, deploy, deploy === "failed" ? "Build failed" : null)] : [];
    },
    async reinstall(machine, build, id) {
      calls.push(`reinstall ${machine} ${build}`);
      if (reinstall instanceof Error) throw reinstall;
      return run(id, "pending");
    },
    async setupBuild(machine) {
      calls.push(`setup ${machine}`);
      const next = builds.shift() ?? "build";
      if (next instanceof Error) throw next;
      return next;
    },
    async serviceHistory(_machine) {
      return [
        run(
          `id-${ids}`,
          reinstall as Operation["outcome"],
          "The plugin is current for Claude Code.",
        ),
      ];
    },
  };
  return { deps, calls };
}

describe("repairPlugin", () => {
  const signal = new AbortController().signal;

  it("deploys first when the dashboard may, then reinstalls and waits for the outcome", async () => {
    const { deps, calls } = fake("succeeded", "succeeded");
    const stages: RepairStage[] = [];
    const result = await repairPlugin(
      { machine: "macbook", deployRevision: "rev", build: "build", restartsDashboard: false },
      deps,
      (stage) => stages.push(stage),
      signal,
    );
    expect(result).toEqual({
      ok: true,
      failed: null,
      message: "The plugin is current for Claude Code.",
    });
    expect(calls).toEqual(["deploy macbook rev", "reinstall macbook build"]);
    expect(stages).toContainEqual({ stage: "updating", detail: "Building" });
    expect(stages.at(-1)).toEqual({ stage: "reinstalling" });
  });

  it("does not reinstall when the update failed", async () => {
    const { deps, calls } = fake("failed", "succeeded");
    const result = await repairPlugin(
      { machine: "macbook", deployRevision: "rev", build: "build", restartsDashboard: false },
      deps,
      () => {},
      signal,
    );
    expect(result.ok).toBe(false);
    expect(result.failed).toBe("update");
    expect(result.message).toContain("Build failed");
    expect(calls).toEqual(["deploy macbook rev"]);
  });

  it("only reinstalls without fleet access, and reports a refused request as the reinstall's", async () => {
    const plain = fake("succeeded", "failed");
    const failed = await repairPlugin(
      { machine: "macbook", deployRevision: null, build: "build", restartsDashboard: false },
      plain.deps,
      () => {},
      signal,
    );
    expect(plain.calls).toEqual(["reinstall macbook build"]);
    expect(failed).toMatchObject({ ok: false, failed: "reinstall" });
    const refused = fake("succeeded", new Error("Service endpoint returned HTTP 400"));
    expect(
      await repairPlugin(
        { machine: "macbook", deployRevision: null, build: "build", restartsDashboard: false },
        refused.deps,
        () => {},
        signal,
      ),
    ).toEqual({ ok: false, failed: "reinstall", message: "Service endpoint returned HTTP 400" });
  });

  it("after a deploy that restarts the dashboard, waits for its new build and reinstalls against it", async () => {
    // The dashboard is away while it restarts, then answers on the old build
    // once more before the new one.
    const { deps, calls } = fake("succeeded", "succeeded", [
      new Error("connection refused"),
      "build",
      "new-build",
    ]);
    const stages: RepairStage[] = [];
    const result = await repairPlugin(
      { machine: "macbook", deployRevision: "rev", build: "build", restartsDashboard: true },
      deps,
      (stage) => stages.push(stage),
      signal,
    );
    expect(result.ok).toBe(true);
    expect(calls).toEqual([
      "deploy macbook rev",
      "setup macbook",
      "setup macbook",
      "setup macbook",
      "reinstall macbook new-build",
    ]);
    expect(stages).toContainEqual({
      stage: "updating",
      detail: "Waiting for the dashboard on macbook to start on the new release.",
    });
  });

  it("does not reinstall when the restarted dashboard never answers", async () => {
    const away = Array.from({ length: 45 }, () => new Error("connection refused"));
    const { deps, calls } = fake("succeeded", "succeeded", away);
    const result = await repairPlugin(
      { machine: "macbook", deployRevision: "rev", build: "build", restartsDashboard: true },
      deps,
      () => {},
      signal,
    );
    expect(result).toMatchObject({ ok: false, failed: "reinstall" });
    expect(result.message).toContain("did not answer within 90 seconds");
    expect(calls.filter((call) => call.startsWith("reinstall"))).toEqual([]);
  });

  it("says the dashboard was left on the old release when the deploy does not restart it", async () => {
    const { deps } = fake("succeeded", new Error("Remote action outcome could not be verified."));
    const result = await repairPlugin(
      { machine: "macbook", deployRevision: "rev", build: "build", restartsDashboard: false },
      deps,
      () => {},
      signal,
    );
    expect(result.failed).toBe("reinstall");
    expect(result.message).toContain("still runs the release from before");
  });

  it("reinstalls against the build a restarted dashboard keeps answering on", async () => {
    const { deps, calls } = fake("succeeded", "succeeded");
    const result = await repairPlugin(
      { machine: "macbook", deployRevision: "rev", build: "build", restartsDashboard: true },
      deps,
      () => {},
      signal,
    );
    expect(result.ok).toBe(true);
    expect(calls.at(-1)).toBe("reinstall macbook build");
  });
});
