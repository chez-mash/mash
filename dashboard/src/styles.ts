// The dashboard's look, in the order the cascade needs: the typefaces, the
// theme's roles, Tailwind's names for them, then the global stylesheets.
// The dashboard's entry point and Storybook's preview both import this, so a
// story is drawn with exactly what the dashboard is drawn with.
import "./theme/fonts";
import "./theme/tokens.css";
import "./app.css";
import "./terminal.css";
import "./kit.css";
import "./semantics.css";
