import type { Item } from "./steps";

/**
 * Whether the item at `at` starts one of the agent's turns, so the agent's
 * name heads it: the first item after something the reader said, or the
 * first item of the page when it is not theirs.
 */
export function startsAgentTurn(items: readonly Item[], at: number): boolean {
  const fromReader = (index: number): boolean => {
    const item = items[index];
    return item?.kind === "message" && item.message.role === "user";
  };
  return at < items.length && !fromReader(at) && (at === 0 || fromReader(at - 1));
}
