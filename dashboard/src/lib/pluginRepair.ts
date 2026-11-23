// The setup warning's fix for a plugin that does not match the installed
// Mash, done from the dashboard: deploy Mash through Fleet manager when this
// dashboard may, then reinstall the plugin on the machine through the service
// action route. The machine's dashboard does the reinstall itself, so it has
// to run the release just deployed: where the deploy restarts it, the fix
// waits for it to come back on a new build and confirms against that build.
// Restarting the agent is left to the person: nothing here can.
import type { FleetJob, FleetRequest, Operation } from "./services.svelte";

export interface RepairPlan {
  machine: string;
  /** The fleet inventory revision, when this dashboard may deploy to the machine. */
  deployRevision: string | null;
  /** The build that checked the machine's setup; the reinstall is refused if it changed. */
  build: string;
  /** Whether a deploy restarts the machine's dashboard on the release it installs. */
  restartsDashboard: boolean;
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
  /** The build the machine's dashboard runs, from its setup check. */
  setupBuild(machine: string, signal: AbortSignal): Promise<string>;
  requestId(): string;
  sleep(ms: number, signal: AbortSignal): Promise<void>;
}

const POLL_MS = 2000;
/** How long a restarted dashboard may take to answer on its new build. */
const RESTART_POLLS = 45;

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

/**
 * The build the machine's dashboard runs once the deploy has restarted it.
 * While it restarts it does not answer at all, which is expected, not a
 * failure. A dashboard that keeps answering on the old build (the release
 * deployed may be the one it runs) is used as it is: its own check refuses
 * the reinstall if the installed Mash changed under it. Null when it never
 * answered.
 */
async function restarted(
  plan: RepairPlan,
  deps: RepairDeps,
  onStage: (stage: RepairStage) => void,
  signal: AbortSignal,
): Promise<string | null> {
  onStage({
    stage: "updating",
    detail: `Waiting for the dashboard on ${plan.machine} to start on the new release.`,
  });
  let answered: string | null = null;
  for (let poll = 0; poll < RESTART_POLLS; poll++) {
    const build = await deps.setupBuild(plan.machine, signal).catch(() => null);
    if (build && build !== plan.build) return build;
    answered = build ?? answered;
    await deps.sleep(POLL_MS, signal);
  }
  return answered;
}

/** Run the fix, reporting each stage. It never throws: a failure is a result. */
export async function repairPlugin(
  plan: RepairPlan,
  deps: RepairDeps,
  onStage: (stage: RepairStage) => void,
  signal: AbortSignal,
): Promise<RepairResult> {
  let part: "update" | "reinstall" = plan.deployRevision ? "update" : "reinstall";
  // The reinstall confirms against the build it reaches: the restarted one after a deploy.
  let build = plan.build;
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
      if (plan.restartsDashboard) {
        const current = await restarted(plan, deps, onStage, signal);
        if (!current)
          return {
            ok: false,
            failed: "reinstall",
            message: `Mash was updated on ${plan.machine}, but its dashboard did not answer within ${(RESTART_POLLS * POLL_MS) / 1000} seconds, so the plugin was not reinstalled. Check that the dashboard there is running, then try again.`,
          };
        build = current;
      }
    }
    part = "reinstall";
    onStage({ stage: "reinstalling" });
    const id = deps.requestId();
    let run: Operation | undefined = await deps.reinstall(plan.machine, build, id);
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
    // A dashboard the deploy left running is on the release from before it,
    // which cannot speak for the new one and may not know the reinstall.
    const stale =
      part === "reinstall" && plan.deployRevision && !plan.restartsDashboard
        ? ` The update does not restart the dashboard on ${plan.machine}, so it still runs the release from before. Restart that dashboard, then try again.`
        : "";
    return { ok: false, failed: part, message: failure(error) + stale };
  }
}
