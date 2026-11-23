/**
 * A path, address or other long value split where a line may break without
 * cutting a name in two: only after a slash or a backslash. Each piece is
 * drawn with a break opportunity before the next, so
 * `/home/sam/.mash/mash.sqlite` wraps as `/home/sam/.mash/` `mash.sqlite`
 * and never leaves a file's extension or a host's last label on a line of
 * its own. Plain words come back whole.
 */
export function wrapPoints(value: string): string[] {
  return value.split(/(?<=[/\\])(?=[^/\\])/).filter(Boolean);
}
