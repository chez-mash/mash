import { mount } from "svelte";
import "./theme/fonts";
import "./theme/tokens.css";
import "./app.css";
import "./terminal.css";

import App from "./App.svelte";
import { installNavigationBoundary } from "./lib/navigation";
import { restoreTheme } from "./lib/theme.svelte";

restoreTheme();
const target = document.getElementById("app");
if (target === null) throw new Error("missing #app element");

const removeNavigationBoundary = installNavigationBoundary(document);
import.meta.hot?.dispose(removeNavigationBoundary);

export default mount(App, { target });
