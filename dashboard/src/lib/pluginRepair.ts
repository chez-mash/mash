// The setup warning's fix for a plugin that does not match the installed
// Mash, done from the dashboard: deploy Mash through Fleet manager when this
// dashboard may, then reinstall the plugin on the machine through the service
// action route. Restarting the agent is left to the person: nothing here can.
import type { FleetJob, FleetRequest, Operation } from "./services.svelte";

export interface RepairPlan {
  machine: string;
  /** The fleet inventory revision, when this dashboard may deploy to the machine. */
  deployRevision: string | null;
  /** The build that checked the machine's setup; the reinstall is refused if it changed. */
  build: string;
}
export type RepairStage = { stage: "updating"; detail: string | null } | { stage: "reinstalling" };
export interface RepairResult {
  ok: boolean;
  /** Which part failed, so the page can say what is still to do by hand. */
  failed: "update" | "reinstall" | null;
  message: string;
}
export interface RepairDeps {
  submitFleetJob(request: FleetRequest): Promise<FleetJob>;
  fleetHistory(signal: AbortSignal): Promise<FleetJob[]>;
  reinstall(machine: string, build: string, requestId: string): Promise<Operation>;
  serviceHistory(machine: string, signal: AbortSignal): Promise<Operation[]>;
  requestId(): string;
  sleep(ms: number, signal: AbortSignal): Promise<void>;
}

const POLL_MS = 2000;

function failure(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

async function update(
  plan: RepairPlan & { deployRevision: string },
  deps: RepairDeps,
  onStage: (stage: RepairStage) => void,
  signal: AbortSignal,
): Promise<RepairResult | null> {
  const request: FleetRequest = {
    id: deps.requestId(),
    action: "deploy",
    targets: [plan.machine],
    revision: plan.deployRevision,
  };
  let job: FleetJob | undefined = await deps.submitFleetJob(request);
  while (job?.outcome === "running") {
    onStage({ stage: "updating", detail: job.targets[0]?.detail ?? null });
    await deps.sleep(POLL_MS, signal);
    job = (await deps.fleetHistory(signal)).find((item) => item.request.id === request.id);
  }
  if (job?.outcome === "succeeded") return null;
  const detail = job?.targets.find((target) => target.detail)?.detail;
  return {
    ok: false,
    failed: "update",
    message: `Updating Mash on ${plan.machine} did not succeed${detail ? `: ${detail}` : "."} The plugin was not reinstalled.`,
  };
}

/** Run the fix, reporting each stage. It never throws: a failure is a result. */
export async function repairPlugin(
  plan: RepairPlan,
  deps: RepairDeps,
  onStage: (stage: RepairStage) => void,
  signal: AbortSignal,
): Promise<RepairResult> {
  let part: "update" | "reinstall" = plan.deployRevision ? "update" : "reinstall";
  try {
    if (plan.deployRevision) {
      onStage({ stage: "updating", detail: null });
      const failed = await update(
        { ...plan, deployRevision: plan.deployRevision },
        deps,
        onStage,
        signal,
      );
      if (failed) return failed;
    }
    part = "reinstall";
    onStage({ stage: "reinstalling" });
    const id = deps.requestId();
    let run: Operation | undefined = await deps.reinstall(plan.machine, plan.build, id);
    while (run?.outcome === "pending") {
      await deps.sleep(POLL_MS, signal);
      run = (await deps.serviceHistory(plan.machine, signal)).find(
        (item) => item.request.request_id === id,
      );
    }
    if (run?.outcome === "succeeded") return { ok: true, failed: null, message: run.message };
    return {
      ok: false,
      failed: "reinstall",
      message:
        run?.message ??
        `${plan.machine} no longer reports the reinstall. Check its agent setup again.`,
    };
  } catch (error) {
    return { ok: false, failed: part, message: failure(error) };
  }
}
