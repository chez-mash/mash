import type { Wire } from "./routes";
import type { Attachment } from "./types";

/** Where a page of messages was read: another machine's own dashboard, or
 * this one with the node and `shared` flag the page was asked for. */
export interface Source {
  readonly peer?: string;
  readonly node?: string;
  readonly shared?: boolean;
}

/** The pictures the server shows inline; every other file is a download. */
const PICTURES = ["image/png", "image/jpeg", "image/gif", "image/webp"];

/** Why no copy of a file was kept, in the reader's words. */
const MISSING: Record<Exclude<Wire["Availability"], "retained">, string> = {
  not_uploaded: "Its machine has not shared the file",
  missing: "The file could not be found",
  not_allowed: "It was outside the folders Mash may keep files from",
  too_large: "It was larger than the 4 MiB Mash keeps",
  invalid: "The file could not be read",
  external: "It was a link, which Mash does not fetch",
};

/** The URL that reads one attachment of `message` from where it was read. */
export function attachmentUrl(
  message: string,
  attachment: string,
  digest: string,
  source: Source,
): string {
  const params = new URLSearchParams({ message, attachment, digest });
  if (source.peer) return `/api/peer/${encodeURIComponent(source.peer)}/api/attachment?${params}`;
  if (source.node) params.set("node", source.node);
  if (source.shared) params.set("shared", "true");
  return `/api/attachment?${params}`;
}

export function attachments(
  message: string,
  rows: readonly Wire["Metadata"][],
  source: Source,
): Attachment[] {
  return rows.map((row) => {
    const kept = row.availability === "retained" && !!row.digest;
    const mediaType = row.media_type.toLowerCase();
    return {
      id: row.id,
      name: row.name || "Attachment",
      mediaType,
      ...(typeof row.bytes === "number" ? { bytes: row.bytes } : {}),
      image: kept && PICTURES.includes(mediaType),
      ...(kept && row.digest
        ? {
            url: attachmentUrl(message, row.id, row.digest, source),
            downloadUrl: `${attachmentUrl(message, row.id, row.digest, source)}&download=true`,
          }
        : {
            missing:
              row.availability === "retained"
                ? MISSING.missing
                : (MISSING[row.availability] ?? "No copy was kept"),
          }),
    };
  });
}

/** What kind of file a media type is, in a word or two: "PDF", "Text". */
export function kind(mediaType: string): string {
  const known: Record<string, string> = {
    "application/pdf": "PDF",
    "application/json": "JSON",
    "application/zip": "ZIP archive",
    "text/plain": "Text",
    "text/markdown": "Markdown",
    "text/csv": "CSV",
    "text/html": "HTML",
    "image/svg+xml": "SVG image",
  };
  const label = known[mediaType];
  if (label) return label;
  const [type, subtype] = mediaType.split("/");
  if (type === "text") return "Text";
  if (type === "image") return subtype ? `${subtype.toUpperCase()} image` : "Image";
  return "File";
}

/** A file's size the way people say it. */
export function size(bytes: number): string {
  if (bytes < 1024) return `${bytes} bytes`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
