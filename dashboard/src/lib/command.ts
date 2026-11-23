/**
 * Commands the dashboard asks a reader to run in a terminal.
 *
 * In a browser the reader installed `mash`, so `mash index` is what they type.
 * The Mash desktop app carries its own `mash` and does not put it on PATH, so
 * there a command names the app's binary in full; the app sets
 * `__MASH_DESKTOP_COMMAND__` to that path, already quoted for the shell.
 */
import { inDesktopApp } from "./shareLink";

/** The command line for `mash <args>`, such as `mashCommand("index")`. */
export function mashCommand(args: string, scope: object = globalThis): string {
  const program = (scope as { __MASH_DESKTOP_COMMAND__?: unknown }).__MASH_DESKTOP_COMMAND__;
  const mash = inDesktopApp(scope) && typeof program === "string" && program ? program : "mash";
  return `${mash} ${args}`;
}
