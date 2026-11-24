import { describe, expect, it } from "vitest";
import { pinnedFirst, sidebarProjects } from "./pins.svelte";

const projects = ["/a", "/b", "/c", "/d"].map((key) => ({ key }));
const keys = (list: { key: string }[]) => list.map((project) => project.key);

describe("pinned projects", () => {
  it("come first, in the order they were pinned, then the rest as they were", () => {
    expect(keys(pinnedFirst(projects, ["/c", "/a"]))).toEqual(["/c", "/a", "/b", "/d"]);
  });

  it("leave out a pin with no chats in view", () => {
    expect(keys(pinnedFirst(projects, ["/gone", "/d"]))).toEqual(["/d", "/a", "/b", "/c"]);
  });

  it("fill the sidebar first and are never cut to make room", () => {
    expect(keys(sidebarProjects(projects, ["/d"], 2))).toEqual(["/d", "/a"]);
    expect(keys(sidebarProjects(projects, ["/d", "/c", "/b"], 2))).toEqual(["/d", "/c", "/b"]);
  });
});
