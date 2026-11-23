// Loaded as a file, not inline: the dashboard's Content Security Policy
// (default-src 'self') blocks inline scripts, which kept this from running.
// The dashboard's code arrives as module scripts. When this computer's
// network changes while they load (joining Wi-Fi, a VPN connecting,
// Docker starting a container), the browser aborts requests in flight,
// even ones to this machine, and a module that never arrives leaves the
// page blank. Reload to fetch it again. One change can abort more than
// one load: a new container's interface comes up and gains its IPv6
// address about a second and a half later, and each is a change. So
// allow a few reloads in a row, then say so rather than stay blank or
// reload forever. src/main.ts clears the count once the app runs.
addEventListener(
  "error",
  (event) => {
    const script = event.target;
    if (!(script instanceof HTMLScriptElement) || script.type !== "module") return;
    const mark = "mash.reloads-after-load-failure";
    let reload = false;
    try {
      const reloads = Number(sessionStorage.getItem(mark) ?? 0);
      if (reloads < 3) {
        sessionStorage.setItem(mark, String(reloads + 1));
        reload = true;
      }
    } catch {
      // Without storage a reload could not count how often it tried.
    }
    if (reload) return location.reload();
    const notice = document.createElement("div");
    notice.setAttribute("role", "alert");
    notice.style.cssText =
      "max-width: 32rem; margin: 3rem auto; padding: 0 1rem; font: 15px/1.5 system-ui, sans-serif";
    const heading = document.createElement("h1");
    heading.textContent = "Mash couldn’t load";
    heading.style.fontSize = "20px";
    const detail = document.createElement("p");
    detail.textContent =
      "Part of the dashboard didn’t arrive, usually because the network changed while the page loaded. Reload to try again.";
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = "Reload";
    button.addEventListener("click", () => location.reload());
    notice.append(heading, detail, button);
    document.getElementById("app")?.replaceChildren(notice);
  },
  true,
);
