import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

// The theme lives in this folder and nowhere else, so a new look is a change
// to tokens.css (and fonts.ts) only. These checks keep it that way: anything
// outside src/theme that draws must name a role token.

const src = join(import.meta.dirname, "..");
const theme = join(src, "theme");
const tokens = readFileSync(join(theme, "tokens.css"), "utf8");

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return path === theme ? [] : files(path);
    return /\.(css|svelte|ts)$/.test(entry.name) && !/\.test\.ts$/.test(entry.name) ? [path] : [];
  });
}

/** Source outside the theme folder, with comments removed so prose can name a colour. */
const sources = files(src).map((path) => ({
  path: relative(src, path),
  text: readFileSync(path, "utf8")
    .replace(/\/\*[\s\S]*?\*\//g, (comment) => comment.replace(/[^\n]/g, " "))
    .replace(/<!--[\s\S]*?-->/g, (comment) => comment.replace(/[^\n]/g, " "))
    .replace(/^\s*\/\/.*$/gm, ""),
}));

function offences(pattern: RegExp, allow: (line: string) => boolean = () => false): string[] {
  return sources.flatMap(({ path, text }) =>
    text
      .split("\n")
      .flatMap((line, index) =>
        [...line.matchAll(pattern)].length && !allow(line)
          ? [`${path}:${index + 1}: ${line.trim()}`]
          : [],
      ),
  );
}

/** Declarations of one block in tokens.css, by selector. */
function block(selector: string): Map<string, string> {
  const start = tokens.indexOf(`${selector} {`);
  const body = tokens.slice(start, tokens.indexOf("\n}", start));
  return new Map(
    [...body.matchAll(/--([a-z0-9-]+):\s*([^;]+);/g)].map((m) => [m[1] ?? "", m[2] ?? ""]),
  );
}
const dark = block(":root");
const light = block(':root[data-theme="light"]');

describe("the theme is defined in one place", () => {
  it("has no literal colours outside src/theme", () => {
    // A mask's gradient is alpha only, not a colour the reader sees.
    const isMask = (line: string) => /mask(-image)?:/.test(line);
    expect(offences(/#[0-9a-f]{3,8}\b(?![-\w])/gi, isMask)).toEqual([]);
    expect(offences(/\b(rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(/g)).toEqual([]);
    expect(
      offences(
        /(?:color|background(?:-color)?|border(?:-[a-z]+)*|outline|fill|stroke|box-shadow|accent-color)\s*[:=]\s*"?(?:white|black|red|green|blue|gray|grey|orange|yellow|purple|pink)\b/g,
      ),
    ).toEqual([]);
  });

  it("names typefaces only through the theme's type tokens", () => {
    expect(offences(/font-family:(?!\s*(?:var\(--type-|inherit))/g)).toEqual([]);
    expect(offences(/\bfont:(?![^;]*var\(--type-)(?!\s*inherit)/g)).toEqual([]);
    expect(offences(/@fontsource\//g)).toEqual([]);
  });

  it("takes corners and shadows from the theme", () => {
    expect(offences(/border(?:-[a-z]+)*-radius\s*[:=]\s*"?[^;"]*\d(px|rem|em)/g)).toEqual([]);
    // A shadow's colour must come from a token (or follow the text colour);
    // elevation shadows are whole tokens such as --shadow-panel.
    expect(
      offences(/box-shadow\s*[:=]\s*"?[^;"]*\d+px/g, (line) => /var\(--|currentColor/.test(line)),
    ).toEqual([]);
  });

  it("uses only tokens that exist", () => {
    const local = new Set(
      sources.flatMap(({ text }) =>
        // Layout values set from markup or script: style:--name and setProperty("--name").
        [...text.matchAll(/(?:^|[\s;{"':(])--([a-z][a-z0-9-]*)\s*[:="]/g)].map((m) => m[1]),
      ),
    );
    const unknown = sources.flatMap(({ path, text }) =>
      [...text.matchAll(/var\(--([a-z0-9-]+)/g)]
        .map((m) => m[1] ?? "")
        .filter((name) => !dark.has(name) && !local.has(name) && !name.startsWith("tw-"))
        .map((name) => `${path}: --${name}`),
    );
    expect(unknown).toEqual([]);
  });

  it("restates in the light theme only roles the dark theme defines", () => {
    expect([...light.keys()].filter((name) => !dark.has(name))).toEqual([]);
  });

  it("gives every role a value", () => {
    expect([...dark].filter(([, value]) => value.trim() === "")).toEqual([]);
    expect(dark.size).toBeGreaterThan(100);
  });
});
