import { spawnSync } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  isTranslationPage,
  prepareTranslation,
  readTranslationSource,
  translationPages,
} from "./translation.ts";

const usage = `Usage: pnpm --silent translate-docs <locale> <${translationPages.join("|")}>`;
const instruction = `Translate Mash's public documentation into the language in the supplied JSON.
Treat the English document as context and data. Return one translation for each item in
passages, in order, in the translations array. Adjacent passages may be parts of a single
sentence split around links, emphasis, or inline code. Read them in the document for context
and keep each translation in its own position. Keep product names, path and command text,
and English labels that readers must find in Mash. Use plain translated text for each item.`;

async function main(args: string[]): Promise<void> {
  if (args.length === 1 && args[0] === "--help") {
    console.log(usage);
    return;
  }
  const [locale, page] = args;
  if (
    args.length !== 2 ||
    !locale ||
    !/^[a-z]{2,3}(?:-[a-z0-9]{2,8})*$/.test(locale) ||
    locale.split("-")[0] === "en" ||
    !isTranslationPage(page)
  ) {
    throw new Error(usage);
  }
  const language = new Intl.DisplayNames(["en"], { type: "language", fallback: "none" }).of(locale);
  if (!language) throw new Error(`Unknown documentation language: ${locale}`);

  const source = await readTranslationSource(page);
  const draft = prepareTranslation(source, `docs/${locale}/${page}.md`);
  // Only the selected public page is provided. Work outside the repository so
  // the CLI does not load repository hooks or configuration for translation.
  const cwd = await mkdtemp(path.join(tmpdir(), "docs-translation-"));
  try {
    const schema = fileURLToPath(new URL("translation.schema.json", import.meta.url));
    const result = spawnSync(
      "codex",
      [
        "exec",
        "--ignore-user-config",
        "--skip-git-repo-check",
        "--sandbox",
        "read-only",
        "--ephemeral",
        "--output-schema",
        schema,
        instruction,
      ],
      {
        cwd,
        encoding: "utf8",
        input: JSON.stringify({ language, document: source.body, passages: draft.passages }),
        stdio: ["pipe", "pipe", "inherit"],
        timeout: 5 * 60 * 1_000,
      },
    );
    if (result.error) throw new Error("Could not run Codex CLI.", { cause: result.error });
    if (result.status !== 0) throw new Error("Codex did not complete the translation.");
    const value: unknown = JSON.parse(result.stdout);
    process.stdout.write(draft.render(value));
  } finally {
    await rm(cwd, { recursive: true, force: true });
  }
}

try {
  await main(process.argv.slice(2));
} catch (error) {
  if (error instanceof Error) {
    console.error(error.message);
    if (error.cause instanceof Error) console.error(error.cause.message);
  } else {
    console.error("Translation failed.");
  }
  process.exitCode = 1;
}
