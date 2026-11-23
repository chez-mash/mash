/**
 * A path, address or other long value split where a line may break without
 * cutting a name in two: after a slash and before a dot, a colon's port, or
 * a query. Each piece is drawn with a break opportunity before the next, so
 * `/home/sam/.mash/mash.sqlite` wraps as `/home/sam/.mash/` `mash.sqlite`
 * rather than `mas` `h.sqlite`. Plain words come back whole.
 */
export function wrapPoints(value: string): string[] {
  return value.split(/(?<=[/\\])(?=[^/\\])|(?<=[A-Za-z_-])(?=[.?&#][^.\d])/).filter(Boolean);
}
