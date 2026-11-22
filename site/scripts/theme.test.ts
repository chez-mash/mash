import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

// The site's theme lives in src/styles/theme.css and nowhere else, so a new
// look is a change to that file (and the fonts in astro.config.mjs). These
// checks keep it that way: any other stylesheet or component that draws must
// name a role token. Starlight's own variables (--sl-*) are allowed: theme.css
// sets its colours and fonts from the roles.

const src = join(import.meta.dirname, "..", "src");
const themePath = join(src, "styles", "theme.css");
const theme = readFileSync(themePath, "utf8");

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return files(path);
    return /\.(css|astro)$/.test(entry.name) && path !== themePath ? [path] : [];
  });
}

/** Styles outside theme.css, with comments removed so prose can name a colour. */
const sources = [join(src, "styles"), join(src, "components")].flatMap(files).map((path) => {
  const text = readFileSync(path, "utf8");
  // In a component, only its <style> blocks draw; markup may hold href="#…".
  const styles = path.endsWith(".astro")
    ? text.replace(/<style[^>]*>([\s\S]*?)<\/style>|[\s\S]/g, (match, css: string | undefined) =>
        css === undefined ? (match === "\n" ? "\n" : " ") : ` ${css} `,
      )
    : text;
  return {
    path: relative(src, path),
    text: styles.replace(/\/\*[\s\S]*?\*\//g, (comment) => comment.replace(/[^\n]/g, " ")),
  };
});

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

/** Roles defined in theme.css, in any block. */
const roles = new Set([...theme.matchAll(/--([a-z0-9-]+)\s*:/g)].map((m) => m[1]));

describe("the site's theme is defined in one place", () => {
  it("has no literal colours outside theme.css", () => {
    const isMask = (line: string) => /mask(-image)?:/.test(line);
    expect(offences(/#[0-9a-f]{3,8}\b(?![-\w])/gi, isMask)).toEqual([]);
    expect(offences(/\b(rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(/g)).toEqual([]);
  });

  it("names typefaces only through the type roles", () => {
    expect(offences(/font-family:(?!\s*(?:var\(--type-|inherit))/g)).toEqual([]);
    expect(offences(/\bfont:(?![^;]*var\(--type-)(?!\s*inherit)/g)).toEqual([]);
  });

  it("takes corners and shadows from the theme", () => {
    expect(offences(/border(?:-[a-z]+)*-radius\s*:\s*[^;]*\d(px|rem|em)/g)).toEqual([]);
    expect(
      offences(/(?:box|text)-shadow\s*:\s*[^;]*\d+(px|rem|em)/g, (line) => /var\(--/.test(line)),
    ).toEqual([]);
  });

  it("uses only roles that exist", () => {
    // Animation values the landing page's scripts set at run time.
    const scripts = readdirSync(src)
      .filter((name) => name.endsWith(".ts"))
      .map((name) => readFileSync(join(src, name), "utf8"));
    const local = new Set([
      ...sources.flatMap(({ text }) =>
        [...text.matchAll(/--([a-z][a-z0-9-]*)\s*:/g)].map((m) => m[1]),
      ),
      ...scripts.flatMap((text) => [...text.matchAll(/"--([a-z][a-z0-9-]*)"/g)].map((m) => m[1])),
    ]);
    const unknown = sources.flatMap(({ path, text }) =>
      // A variable read with a fallback, var(--name, value), may be left unset.
      [...text.matchAll(/var\(--([a-z0-9-]+)(\s*,)?/g)]
        .filter((m) => m[2] === undefined)
        .map((m) => m[1] ?? "")
        .filter((name) => !roles.has(name) && !local.has(name) && !name.startsWith("sl-"))
        .map((name) => `${path}: --${name}`),
    );
    expect(unknown).toEqual([]);
  });

  it("colours Starlight from the roles", () => {
    const starlight = theme.slice(theme.indexOf("/* Starlight's variables"));
    expect(
      [...starlight.matchAll(/--sl-[a-z0-9-]+:\s*([^;]+);/g)].map((m) => m[1]?.trim()),
    ).toSatisfy(
      (values: string[]) =>
        values.length > 10 && values.every((value) => value.startsWith("var(--")),
    );
  });
});
