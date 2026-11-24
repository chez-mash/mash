import { describe, expect, it } from "vitest";
import { pinnedFirst, sidebarProjects, withoutHidden } from "./projectChoices.svelte";

const projects = ["/a", "/b", "/c", "/d"].map((key) => ({ key }));
const keys = (list: { key: string }[]) => list.map((project) => project.key);

describe("project choices", () => {
  it("put pinned projects first, in the order they were pinned, then the rest", () => {
    expect(keys(pinnedFirst(projects, ["/c", "/a"]))).toEqual(["/c", "/a", "/b", "/d"]);
  });

  it("leave out a pin with no chats in view", () => {
    expect(keys(pinnedFirst(projects, ["/gone", "/d"]))).toEqual(["/d", "/a", "/b", "/c"]);
  });

  it("leave hidden projects out of the list", () => {
    expect(keys(withoutHidden(projects, ["/b"]))).toEqual(["/a", "/c", "/d"]);
  });

  it("fill the sidebar with pins first, never cut them, and skip hidden projects", () => {
    expect(keys(sidebarProjects(projects, { pinned: ["/d"], hidden: [] }, 2))).toEqual([
      "/d",
      "/a",
    ]);
    expect(keys(sidebarProjects(projects, { pinned: ["/d", "/c", "/b"], hidden: [] }, 2))).toEqual([
      "/d",
      "/c",
      "/b",
    ]);
    expect(keys(sidebarProjects(projects, { pinned: [], hidden: ["/a", "/b"] }, 2))).toEqual([
      "/c",
      "/d",
    ]);
  });
});
