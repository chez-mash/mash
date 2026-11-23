import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { missingStylesheets } from "./stylesheets.ts";

let dist = "";

function write(path: string, text: string) {
  const full = join(dist, path);
  mkdirSync(join(full, ".."), { recursive: true });
  writeFileSync(full, text);
}

describe("missingStylesheets", () => {
  afterEach(() => rmSync(dist, { recursive: true, force: true }));

  it("accepts pages whose stylesheets were written", () => {
    dist = mkdtempSync(join(tmpdir(), "mash-dist-"));
    write("_astro/ec.abcde.css", "");
    write("quick-start/index.html", '<link rel="stylesheet" href="/_astro/ec.abcde.css">');
    expect(missingStylesheets(dist)).toEqual([]);
  });

  it("names a page that links a stylesheet the build did not write", () => {
    dist = mkdtempSync(join(tmpdir(), "mash-dist-"));
    write("_astro/ec.abcde.css", "");
    write("dashboard/index.html", '<link rel="stylesheet" href="/_astro/ec.stale.css">');
    expect(missingStylesheets(dist)).toEqual(["dashboard/index.html: /_astro/ec.stale.css"]);
  });
});
