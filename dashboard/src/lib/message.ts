import type { Message } from "./types";

/** Everything a message holds, as text: its content, then its tool records. */
export function messageText(message: Message): string {
  return (
    message.blocks
      .map((block) =>
        block.type === "text" ? block.markdown : block.type === "code" ? block.code : "",
      )
      .join("\n\n") + (message.activity ? `\n\n${message.activity}` : "")
  );
}

/** What was said: a person's own words when the agent added context around them. */
export function saidText(message: Message): string {
  return message.dialogue ?? messageText(message);
}

/** A message's text in the order the page shows it: what the person typed,
 * then the whole message it arrived in. */
export function shownText(message: Message): string {
  const whole = messageText(message);
  return message.dialogue === undefined ? whole : `${message.dialogue}\n\n${whole}`;
}

/** Tool calls, tool output, thinking and context: the work between what was said. */
export function isStep(message: Message): boolean {
  return message.presentation !== undefined && message.presentation !== "dialogue";
}
