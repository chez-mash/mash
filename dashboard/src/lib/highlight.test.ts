import { expect, test } from "vitest";
import { highlightParts } from "./highlight";

test("a match at the start of the text is the part that is marked", () => {
  expect(highlightParts("foo bar", "foo")).toEqual([
    { text: "foo", hit: true },
    { text: " bar", hit: false },
  ]);
});

test("a match marks the whole word it falls in, and trailing matches keep their marks", () => {
  expect(highlightParts("foofoo x bar", "foo bar")).toEqual([
    { text: "foofoo", hit: true },
    { text: " x ", hit: false },
    { text: "bar", hit: true },
  ]);
  expect(highlightParts("the queues ran", "queue")).toEqual([
    { text: "the ", hit: false },
    { text: "queues", hit: true },
    { text: " ran", hit: false },
  ]);
});

test("words matched next to each other are one mark", () => {
  expect(highlightParts("so invoice exports run, invoice. exports", "invoice exports")).toEqual([
    { text: "so ", hit: false },
    { text: "invoice exports", hit: true },
    { text: " run, ", hit: false },
    { text: "invoice", hit: true },
    { text: ". ", hit: false },
    { text: "exports", hit: true },
  ]);
});

test("matching ignores case, treats the query as text and skips one-letter words", () => {
  expect(highlightParts("A.b a+b", "a A.B")).toEqual([
    { text: "A.b", hit: true },
    { text: " a+b", hit: false },
  ]);
  expect(highlightParts("plain", "p")).toEqual([{ text: "plain", hit: false }]);
});
