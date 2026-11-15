import { expect, test } from "vitest";
import { highlightParts } from "./highlight";

test("a match at the start of the text is the part that is marked", () => {
  expect(highlightParts("foo bar", "foo")).toEqual([
    { text: "foo", hit: true },
    { text: " bar", hit: false },
  ]);
});

test("adjacent and trailing matches keep their marks", () => {
  expect(highlightParts("foofoo x bar", "foo bar")).toEqual([
    { text: "foo", hit: true },
    { text: "foo", hit: true },
    { text: " x ", hit: false },
    { text: "bar", hit: true },
  ]);
});

test("matching ignores case, treats the query as text and skips one-letter words", () => {
  expect(highlightParts("A.b a+b", "a A.B")).toEqual([
    { text: "A.b", hit: true },
    { text: " a+b", hit: false },
  ]);
  expect(highlightParts("plain", "p")).toEqual([{ text: "plain", hit: false }]);
});
