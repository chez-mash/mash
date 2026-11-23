// `make site-review`: the Astro dev server plus the Agentation services that
// turn annotations left on the page into queued agent work.
import { defineConfig } from "agentation-review";
import { reviewPorts } from "./src/agentation.ts";

export default defineConfig({
  projectDirectory: "..",
  projectName: "mash",
  runtimeDirectory: ".agentation-runtime",
  availableAgents: ["codex", "claude"],
  defaultAgent: "claude",
  sync: { port: reviewPorts.syncPort },
  review: { port: reviewPorts.reviewPort },
  devServer: {
    command: "astro",
    args: ["dev", "--host", "127.0.0.1", "--port", "4331"],
    cwd: ".",
    env: {
      // Astro otherwise detaches into the background in detected AI
      // environments. In the foreground, agentation-review owns the process.
      ASTRO_DEV_BACKGROUND: "1",
    },
  },
  codex: { approveForMe: true },
  claude: { dangerouslySkipPermissions: true },
  workerInstructions:
    "Do not commit or push. Do not modify the Agentation review integration unless the feedback explicitly concerns it.",
});
