// Which system each machine runs, for the logo beside a machine's name on any
// page. It asks the same two questions as the status marks (this dashboard's
// service checks and the shared server's report) and shares their cache, but
// never on a timer: the answer is read once, kept, and refreshed only when a
// page that shows the machines' state reads it again. So a logo in the rail or
// in a chat's meta line adds no background requests, and a failed read is not
// retried until the reader asks for an update.
import { createQuery } from "@tanstack/svelte-query";
import { machineOs } from "./os";
import { stableRead } from "./queries";
import { serverReport, serverViewOptions } from "./serverStatus";
import { serviceAccessOptions, serviceOptions } from "./services.svelte";

const once = { ...stableRead, retryOnMount: false } as const;

export function createMachineOs() {
  const access = createQuery(() => ({ ...serviceAccessOptions, ...once }));
  const fleet = createQuery(() => ({ ...serviceOptions(access.data !== undefined), ...once }));
  const server = createQuery(() => ({ ...serverViewOptions, ...once }));
  return {
    of(machine: string) {
      return machineOs(machine, fleet.data, serverReport(server.data));
    },
  };
}
