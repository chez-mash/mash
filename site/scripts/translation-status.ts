import { readFile } from "node:fs/promises";
import { docsLocales } from "./locales.ts";
import {
  readTranslationSource,
  type TranslationStatus,
  translationPages,
  translationStatus,
} from "./translation.ts";

const repository = new URL("../../", import.meta.url);
const statusLabels: Record<Exclude<TranslationStatus, "current">, string> = {
  outdated: "English source changed",
  untracked: "No source revision recorded",
  missing: "Translation missing",
};

export async function translationReport(selectedLocale?: string): Promise<string> {
  const locales = Object.entries(docsLocales).filter(([locale]) => locale !== "root");
  const selected = locales.filter(
    ([locale]) => selectedLocale === undefined || locale === selectedLocale,
  );
  if (!selected.length) {
    const choices = locales.map(([locale]) => locale).join(", ");
    throw new Error(`Unknown published translation language. Choose ${choices}.`);
  }

  const sources = await Promise.all(
    translationPages.map(async (page) => ({
      page,
      revision: (await readTranslationSource(page)).revision,
    })),
  );
  const lines = [
    "## Documentation translation status",
    "",
    "Guides supported by the translation tool. Review the prose and links before recording a new source revision.",
    "",
    "| Language | Source matches | Source changed | No source revision | Missing |",
    "| --- | ---: | ---: | ---: | ---: |",
  ];
  const review: string[] = [];
  for (const [locale, { label }] of selected) {
    const counts: Record<TranslationStatus, number> = {
      current: 0,
      outdated: 0,
      untracked: 0,
      missing: 0,
    };
    for (const { page, revision } of sources) {
      const file = `docs/${locale}/${page}.md`;
      const body = await readFile(new URL(file, repository), "utf8").catch((error: unknown) => {
        if (error instanceof Error && "code" in error && error.code === "ENOENT") return undefined;
        throw error;
      });
      const status = translationStatus(revision, body);
      counts[status] += 1;
      if (status === "current") continue;
      const action =
        status === "untracked"
          ? ""
          : ` · Draft with \`pnpm --silent translate-docs ${locale} ${page}\``;
      review.push(`- \`${file}\` · ${statusLabels[status]}${action}`);
    }
    lines.push(
      `| ${label} (${locale}) | ${counts.current} | ${counts.outdated} | ${counts.untracked} | ${counts.missing} |`,
    );
  }
  return lines
    .concat("", review.length ? review : "All checked translations have matching source revisions.")
    .join("\n");
}
