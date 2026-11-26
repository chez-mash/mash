// The theme's roles as the foundations stories list them: read from
// tokens.css itself, so a role added to the theme shows up without a story
// being edited.
import source from "../theme/tokens.css?raw";

export interface Role {
  /** The custom property's name, such as "--surface-page". */
  readonly name: string;
  /** The value written for the light theme. */
  readonly written: string;
}

export interface RoleGroup {
  /** What the roles share: the part of the name before its first dash. */
  readonly family: string;
  readonly roles: readonly Role[];
}

/** Every role the light theme declares, in the order tokens.css writes them. */
export function roles(): Role[] {
  const start = source.indexOf(":root {");
  const body = source.slice(start, source.indexOf("\n}", start));
  return [...body.matchAll(/(--[a-z0-9-]+):\s*([^;]+);/g)].map((found) => ({
    name: found[1] ?? "",
    written: (found[2] ?? "").replace(/\s+/g, " "),
  }));
}

/** Roles gathered by family, keeping the order each family first appears in. */
export function grouped(list: readonly Role[]): RoleGroup[] {
  const groups = new Map<string, Role[]>();
  for (const role of list) {
    const family = role.name.slice(2).split("-")[0] ?? "";
    groups.set(family, [...(groups.get(family) ?? []), role]);
  }
  return [...groups].map(([family, members]) => ({ family, roles: members }));
}

/** Whether a role's value, as the browser resolved it, is a colour. */
export function isColour(name: string, root: Element = document.documentElement): boolean {
  const value = getComputedStyle(root).getPropertyValue(name).trim();
  return value !== "" && CSS.supports("color", value);
}
