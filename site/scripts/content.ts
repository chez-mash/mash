import { copyFile, cp, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { docsLocales } from "./locales.ts";
import { readmeQuickStart, renderMarkdown } from "./markdown.ts";
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
    tipBefore: word("mash.guides.tipBefore"),
    tipAfter: word("mash.guides.tipAfter"),
    copy: word("mash.copy"),
    copied: word("mash.copied"),
    copyPrompt: word("mash.copyPrompt"),
  };
}
const architectureImage = "mash-architecture.svg";
const markdownRoutes = new Map(routes).set(`assets/${architectureImage}`, `/${architectureImage}`);
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
for (const { name: locale } of localeFolders) await mkdir(new URL(`${locale}/`, output));
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
  const title = source === "README.md" ? "Quick start" : body.match(/^# (.+)$/m)?.[1];
  if (!title) throw new Error(`Missing title in ${source}`);
  if (source === "README.md") {
    body = `<span id="quick-start"></span>\n\n${readmeQuickStart(body)}`;
  } else body = body.replace(/^# .+\n/m, "");
  const filename = source.endsWith("/index.md") ? `${route.slice(1)}index` : route.slice(1, -1);
  // The guides directory is a page of cards across the full width.
  const template = page === "guides" ? "template: splash\n" : "";
  const frontmatter = `---\ntitle: ${JSON.stringify(title)}\neditUrl: ${JSON.stringify(`https://github.com/mashium/mash/edit/main/${source}`)}\n${template}---\n`;
  const locale = route.split("/")[1];
  const layout = {
    page: page ?? "",
    strings: await layoutStrings(locales.has(locale ?? "") ? locale : undefined),
  };
  await writeFile(
    new URL(`${filename}.md`, output),
    frontmatter + renderMarkdown(body, source, markdownRoutes, layout),
  );
}
await copyFile(new URL("docs/home.mdx", repo), new URL("index.mdx", output));
await copyFile(
  new URL(`assets/${architectureImage}`, repo),
  new URL(`../public/${architectureImage}`, import.meta.url),
);
// The landing page shows the dashboard's own agent marks and its mascot.
await copyFile(
  new URL("dashboard/public/mashy.png", repo),
  new URL("../public/mashy.png", import.meta.url),
);
await cp(new URL("dashboard/public/agents/", repo), new URL("../public/agents/", import.meta.url), {
  recursive: true,
});
console.log(`Prepared documentation in ${fileURLToPath(output)}`);
