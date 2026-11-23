import { mount } from "svelte";
import "./theme/fonts";
import "./theme/tokens.css";
import "./app.css";
import "./terminal.css";
import "./kit.css";

import App from "./App.svelte";
import { installNavigationBoundary } from "./lib/navigation";

const target = document.getElementById("app");
if (target === null) throw new Error("missing #app element");

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
