// The app opens this page with ?error=… when the Mash engine could not start
// or stopped; otherwise it replaces the page with the dashboard once ready.
const error = new URLSearchParams(location.search).get("error");
if (error) {
  document.getElementById("status").textContent = "Mash couldn’t start.";
  const detail = document.getElementById("detail");
  detail.textContent = `${error} Quit and reopen Mash to try again; if this keeps happening, run “mash doctor” in a terminal.`;
  detail.hidden = false;
}
