import { cp, mkdir, readdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { docsLocales } from "./locales.ts";
import { readmeQuickStart, renderMarkdown, spaceFrenchPunctuation } from "./markdown.ts";
import {
  isTranslationPage,
  readTranslationSource,
  recordedTranslationRevision,
} from "./translation.ts";

const repo = new URL("../../", import.meta.url);
const output = new URL("../src/content/docs/", import.meta.url);
const names = (await readdir(new URL("docs/", repo))).filter((name) => name.endsWith(".md"));
const routes = new Map(
  names.map((name) => [
    `docs/${name}`,
    name === "README.md" ? "/guides/" : `/${name.slice(0, -3)}/`,
  ]),
);
const locales = new Set(Object.keys(docsLocales).filter((locale) => locale !== "root"));
const localeFolders = (await readdir(new URL("docs/", repo), { withFileTypes: true })).filter(
  (entry) => entry.isDirectory() && locales.has(entry.name),
);
for (const { name: locale } of localeFolders) {
  for (const name of await readdir(new URL(`docs/${locale}/`, repo))) {
    if (!name.endsWith(".md")) continue;
    const route = name === "index.md" ? `/${locale}/` : `/${locale}/${name.slice(0, -3)}/`;
    routes.set(`docs/${locale}/${name}`, route);
  }
}
routes.set("README.md", "/quick-start/");
routes.set("docs/home.mdx", "/");
/** The words the page layouts add, from the site's own translations. */
async function layoutStrings(locale: string | undefined) {
  const languages: Readonly<Record<string, { label: string; lang?: string }>> = docsLocales;
  const lang = locale ? (languages[locale]?.lang ?? locale) : "en";
  const read = async (name: string): Promise<Record<string, string>> =>
    JSON.parse(
      await readFile(new URL(`../src/content/i18n/${name}.json`, import.meta.url), "utf8"),
    );
  const strings = { ...(await read("en")), ...(locale ? await read(lang) : {}) };
  const word = (key: string): string => strings[key] ?? "";
  return {
    tip: word("mash.guides.tip"),
    tipSearch: word("mash.guides.tipSearch"),
    copy: word("mash.copy"),
    copied: word("mash.copied"),
    copyPrompt: word("mash.copyPrompt"),
    no: word("mash.answer.no"),
  };
}
// Diagrams the guides link from the repository's assets directory.
const diagrams = ["mash-architecture.svg", "mash-shared-index.svg"];
const markdownRoutes = new Map(routes);
for (const diagram of diagrams) markdownRoutes.set(`assets/${diagram}`, `/${diagram}`);
// Pictures of the dashboard the guides show, captured by scripts/screenshots.ts.
const screenshots = (await readdir(new URL("assets/screenshots/", repo))).filter((file) =>
  file.endsWith(".webp"),
);
for (const file of screenshots)
  markdownRoutes.set(`assets/screenshots/${file}`, `/screenshots/${file}`);
// Every page is rendered first and then synced into place, never by emptying
// the folder: a running `astro dev` watches it, and a folder caught half empty
// makes Starlight fail on a sidebar page it can no longer find.
const pages = new Map<string, string>();
for (const [source, route] of routes) {
  if (source === "docs/home.mdx") continue;
  let body = await readFile(new URL(source, repo), "utf8");
  const revision = recordedTranslationRevision(body);
  const page = route.split("/").at(-2);
  if (
    revision &&
    isTranslationPage(page) &&
    revision !== (await readTranslationSource(page)).revision
  ) {
    console.warn(`${source} needs review because its English source changed.`);
  }
  const heading = source === "README.md" ? "Quick start" : body.match(/^# (.+)$/m)?.[1];
  if (!heading) throw new Error(`Missing title in ${source}`);
  const title = source.startsWith("docs/fr/") ? spaceFrenchPunctuation(heading) : heading;
  if (source === "README.md") {
    body = `<span id="quick-start"></span>\n\n${readmeQuickStart(body)}`;
  } else body = body.replace(/^# .+\n/m, "");
  const filename = source.endsWith("/index.md") ? `${route.slice(1)}index` : route.slice(1, -1);
  // The guides directory is a page of cards across the full width.
  const template = page === "guides" ? "template: splash\n" : "";
  const frontmatter = `---\ntitle: ${JSON.stringify(title)}\neditUrl: ${JSON.stringify(`https://github.com/mashhq/mash/edit/main/${source}`)}\n${template}---\n`;
  const locale = route.split("/")[1];
  const layout = {
    page: page ?? "",
    strings: await layoutStrings(locales.has(locale ?? "") ? locale : undefined),
  };
  pages.set(`${filename}.md`, frontmatter + renderMarkdown(body, source, markdownRoutes, layout));
}
pages.set("index.mdx", await readFile(new URL("docs/home.mdx", repo), "utf8"));
await syncFolder(fileURLToPath(output), pages);
for (const diagram of diagrams) {
  await writeIfChanged(
    fileURLToPath(new URL(`../public/${diagram}`, import.meta.url)),
    await readFile(new URL(`assets/${diagram}`, repo)),
  );
}
await cp(new URL("assets/screenshots/", repo), new URL("../public/screenshots/", import.meta.url), {
  recursive: true,
});
// The landing page shows the dashboard's own agent marks and its mascot.
await writeIfChanged(
  fileURLToPath(new URL("../public/mashy.png", import.meta.url)),
  await readFile(new URL("dashboard/public/mashy.png", repo)),
);
await cp(new URL("dashboard/public/agents/", repo), new URL("../public/agents/", import.meta.url), {
  recursive: true,
});
console.log(`Prepared documentation in ${fileURLToPath(output)}`);

/** Writes `contents` only when it differs, through a temporary file renamed
 * into place, so a reader never sees a half-written file and an unchanged
 * page is not touched (and does not trigger a reload). */
async function writeIfChanged(path: string, contents: string | Buffer): Promise<boolean> {
  const next = typeof contents === "string" ? Buffer.from(contents) : contents;
  const current = await readFile(path).catch(() => undefined);
  if (current?.equals(next)) return false;
  await mkdir(dirname(path), { recursive: true });
  const temporary = `${path}.${process.pid}.tmp`;
  await writeFile(temporary, next);
  await rename(temporary, path);
  return true;
}

/** Makes `folder` hold exactly `files` (relative path to contents): changed
 * files are replaced, new ones added, and only then are pages that no longer
 * exist removed. */
async function syncFolder(folder: string, files: ReadonlyMap<string, string>): Promise<void> {
  for (const [name, contents] of files) await writeIfChanged(join(folder, name), contents);
  const existing = await readdir(folder, { recursive: true, withFileTypes: true }).catch(() => []);
  for (const entry of existing) {
    if (!entry.isFile()) continue;
    const name = relative(folder, join(entry.parentPath, entry.name));
    if (!files.has(name)) await rm(join(folder, name), { force: true });
  }
}
