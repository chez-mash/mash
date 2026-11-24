// What the dashboard knows about the machines, for the marks drawn beside a
// machine's name: this dashboard's own service checks and the shared server's
// report. Both repeat every minute while a mark is on screen, so what the
// marks say stays current; every mark shares the same two queries.
import { createQuery } from "@tanstack/svelte-query";
import { serverReport, serverViewOptions } from "./serverStatus";
import { serviceAccessOptions, serviceOptions } from "./services.svelte";

const EVERY_MINUTE = 60_000;

export function createMachineEvidence() {
  const access = createQuery(() => serviceAccessOptions);
  const fleet = createQuery(() => ({
    ...serviceOptions(access.data !== undefined),
    refetchInterval: EVERY_MINUTE,
  }));
  const server = createQuery(() => ({ ...serverViewOptions, refetchInterval: EVERY_MINUTE }));
  return {
    get fleet() {
      return fleet.data;
    },
    get report() {
      return serverReport(server.data);
    },
    /** When the service checks were read, for "checked 2m ago". */
    get now() {
      return fleet.dataUpdatedAt || Date.now();
    },
  };
}
