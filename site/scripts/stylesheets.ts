// Checks that every stylesheet a built page links was written to dist.
//
// Astro keeps each guide's rendered HTML in its content cache and reuses it
// while the Markdown is unchanged. Expressive Code names its stylesheet after
// a hash of the code-block styles, and its options sit inside the Starlight
// integration, which Astro leaves out of the config digest that clears the
// cache. So after a style change, a guide whose text had not changed kept a
// link to the old ec.*.css while the build wrote only the new one, and its
// code blocks lost their styles. `pnpm build` now clears the cache (--force)
// and then runs this check, so a stale link fails the build instead.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

function pages(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return pages(path);
    return entry.name.endsWith(".html") ? [path] : [];
  });
}

/** The /_astro/*.css links in built pages whose files are missing, as "page: href". */
export function missingStylesheets(dist: string): string[] {
  const missing: string[] = [];
  for (const page of pages(dist)) {
    const html = readFileSync(page, "utf8");
    for (const [, href] of html.matchAll(/href="(\/_astro\/[^"?#]+\.css)"/g)) {
      if (!existsSync(join(dist, href))) missing.push(`${relative(dist, page)}: ${href}`);
    }
  }
  return missing;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const dist = fileURLToPath(new URL("../dist/", import.meta.url));
  const missing = missingStylesheets(dist);
  if (missing.length > 0) {
    console.error(`Built pages link stylesheets the build did not write:\n${missing.join("\n")}`);
    process.exitCode = 1;
  } else {
    console.log("Every stylesheet the built pages link is in dist.");
  }
}
