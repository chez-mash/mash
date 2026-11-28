// The app opens this page with ?error=… when the Mash engine could not start
// or stopped; otherwise it replaces the page with the dashboard once ready.
// The page may ask the app for two things only (capabilities/default.json):
// to start the engine again, and for the engine's last lines of output.
const error = new URLSearchParams(location.search).get("error");
const ask = (command) => window.__TAURI_INTERNALS__.invoke(command);
// How to run the app's own mash in a terminal (set by the app, cli.rs).
const mash = window.__MASH_DESKTOP_COMMAND__ || "mash";

const status = document.getElementById("status");
const detail = document.getElementById("detail");
const actions = document.getElementById("actions");
const retry = document.getElementById("retry");
const output = document.getElementById("output");
const lines = document.getElementById("lines");
const copy = document.getElementById("copy");

if (error) {
  status.textContent = "Mash isn’t running.";
  detail.textContent = `${error} Try again; if this keeps happening, run “${mash} doctor” in a terminal.`;
  detail.hidden = false;
  actions.hidden = false;
  ask("engine_log").then(
    (log) => {
      if (log.length === 0) return;
      lines.textContent = log.join("\n");
      output.hidden = false;
    },
    () => {},
  );
}

retry.addEventListener("click", () => {
  status.textContent = "Starting Mash…";
  detail.hidden = true;
  actions.hidden = true;
  output.hidden = true;
  // The app shows the dashboard when the engine is ready, or this page with
  // the new error.
  ask("retry_engine").catch(() => {
    status.textContent = "Mash isn’t running.";
    actions.hidden = false;
  });
});

copy.addEventListener("click", async () => {
  const text = lines.textContent;
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    // Older webviews: copy the selected text instead.
    const range = document.createRange();
    range.selectNodeContents(lines);
    const selection = getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
    document.execCommand("copy");
    selection.removeAllRanges();
  }
  copy.textContent = "Copied";
  setTimeout(() => {
    copy.textContent = "Copy";
  }, 1500);
});

// A right-click here opens nothing, as in a native app and on the dashboard
// (dashboard/src/lib/contextMenu.svelte.ts): the webview's own page menu
// (Back, Refresh, Print) has nothing to offer. Selected text keeps the
// system's menu, for copying the engine's output.
addEventListener("contextmenu", (event) => {
  const selection = getSelection();
  if (selection && !selection.isCollapsed && selection.toString().trim() !== "") return;
  event.preventDefault();
});
