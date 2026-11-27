import type { Preview } from "@storybook/svelte-vite";
import "../src/styles";
// The dashboard ships one stylesheet, so what a transcript reads with is on
// every page of it; Mashy's answers and a saved memory's card are drawn with it.
import "../src/components/reading.css";
import { installIconTheme } from "../src/lib/icon.svelte";
import { installEvents } from "../src/lib/transport";
import type { StoryData } from "../src/stories/answers";
import Frame from "../src/stories/Frame.svelte";

// Made-up history never changes, so there is no event stream to listen to.
installEvents(undefined);
installIconTheme();

const preview: Preview = {
  parameters: {
    layout: "padded",
    // The page behind a story is the theme's own surface, not a swatch.
    backgrounds: { disable: true },
    viewport: {
      options: {
        desktop: { name: "Desktop 1280", styles: { width: "1280px", height: "860px" } },
        phone: { name: "Phone 390", styles: { width: "390px", height: "844px" } },
      },
    },
    controls: { expanded: false },
    options: {
      storySort: { order: ["Foundations", "Kit", "Blocks", "Composed"] },
    },
  },
  decorators: [
    (_, context) => {
      const data = context.parameters.data as StoryData | undefined;
      return { Component: Frame, props: { data } };
    },
  ],
};

export default preview;
