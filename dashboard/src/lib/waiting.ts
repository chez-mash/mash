/**
 * What Mashy says while it looks things up. The first is plain, so a quick
 * answer only ever shows that; the rest appear when it takes longer.
 */
export const WAITING = [
  "Searching your history…",
  "Reading old chats…",
  "Asking past you…",
  "Checking what the agents said…",
  "Following the paper trail…",
  "Dusting off last week…",
  "Lifting the lid…",
  "Mashing it together…",
] as const;

/** How long each phrase stays, in milliseconds. */
export const WAITING_INTERVAL = 2500;

/** The phrase for the nth interval of one wait; they come round again. */
export function waitingPhrase(step: number): string {
  const place = ((Math.trunc(step) % WAITING.length) + WAITING.length) % WAITING.length;
  return WAITING[place] ?? WAITING[0];
}
