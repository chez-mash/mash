import { mount } from "svelte";
import "./styles";

import App from "./App.svelte";
import { installContextMenu } from "./lib/contextMenu.svelte";
import { installIconTheme } from "./lib/icon.svelte";
import { installNavigationBoundary } from "./lib/navigation";
import { installShell } from "./lib/shell";

const target = document.getElementById("app");
if (target === null) throw new Error("missing #app element");

installShell();
installContextMenu();
installIconTheme();

const removeNavigationBoundary = installNavigationBoundary(document);
import.meta.hot?.dispose(removeNavigationBoundary);

const app = mount(App, { target });
// The page's code arrived, so a later failed load starts its reload count
// afresh (see the load-failure handler in index.html).
try {
  sessionStorage.removeItem("mash.reloads-after-load-failure");
} catch {
  // Storage is unavailable, so the handler never counted a reload either.
}
export default app;
