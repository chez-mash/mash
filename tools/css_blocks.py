"""Read a stylesheet as nested blocks, and a selector as the class and id
names it mentions. Enough CSS for `unused_css.py`: comments, strings, at-rule
blocks and nested rules are understood; declarations are not interpreted."""

from __future__ import annotations

import re
from dataclasses import dataclass, field

NAME = r"-?[A-Za-z_][\w-]*"
SELECTOR_NAME = re.compile(rf"([.#])({NAME})")
OPAQUE_PSEUDO = re.compile(r":(?:not|is|where|has|global)\(")
OPAQUE_AT_RULES = (
    "@keyframes",
    "@font-face",
    "@theme",
    "@property",
    "@counter-style",
    "@font-feature-values",
)


@dataclass
class Rule:
    """A style rule or an at-rule block, with offsets into its stylesheet."""

    prelude: str
    start: int  # of the prelude
    open: int  # of `{`
    end: int  # just past `}`
    parent: Rule | None = None
    children: list[Rule] = field(default_factory=list)
    declarations: bool = False  # whether the block holds anything but rules

    @property
    def at_rule(self) -> bool:
        return self.prelude.startswith("@")

    @property
    def opaque(self) -> bool:
        return self.prelude.startswith(OPAQUE_AT_RULES)


def blank_comments(css: str) -> str:
    """The stylesheet with each comment replaced by spaces, offsets unchanged."""
    return re.sub(
        r"/\*.*?\*/",
        lambda found: re.sub(r"[^\n]", " ", found.group()),
        css,
        flags=re.S,
    )


def parse(css: str) -> list[Rule]:
    """The top-level blocks of a stylesheet, nested blocks as their children."""
    text = blank_comments(css)
    top: list[Rule] = []
    current: Rule | None = None
    statement = 0
    quote = ""
    for index, char in enumerate(text):
        if quote:
            if char == quote and text[index - 1] != "\\":
                quote = ""
        elif char in "\"'":
            quote = char
        elif char == "{":
            start = (
                statement
                + len(text[statement:index])
                - len(text[statement:index].lstrip())
            )
            rule = Rule(" ".join(text[start:index].split()), start, index, -1, current)
            (current.children if current else top).append(rule)
            current = rule
            statement = index + 1
        elif char == ";":
            if current and text[statement:index].strip():
                current.declarations = True
            statement = index + 1
        elif char == "}" and current:
            if text[statement:index].strip():
                current.declarations = True
            current.end = index + 1
            current = current.parent
            statement = index + 1
    return top


def split_selectors(prelude: str) -> list[str]:
    """A selector list split at its top-level commas."""
    parts, depth, start = [], 0, 0
    for index, char in enumerate(prelude):
        if char in "([":
            depth += 1
        elif char in ")]":
            depth -= 1
        elif char == "," and depth == 0:
            parts.append(prelude[start:index].strip())
            start = index + 1
    parts.append(prelude[start:].strip())
    return [part for part in parts if part]


def without_brackets(selector: str, pattern: re.Pattern[str] | None) -> str:
    """A selector without attribute values and, with a pattern, without the
    contents of the pseudo-classes it matches."""
    selector = re.sub(r"\[[^\]]*\]", "[]", selector)
    if pattern is None:
        return selector
    while found := pattern.search(selector):
        depth, index = 1, found.end()
        while index < len(selector) and depth:
            depth += {"(": 1, ")": -1}.get(selector[index], 0)
            index += 1
        selector = selector[: found.start()] + selector[index:]
    return selector


def names_in(selector: str, required_only: bool = False) -> set[tuple[str, str]]:
    """The (sigil, name) pairs a selector mentions; with `required_only`, only
    those an element must carry for the selector to match."""
    stripped = without_brackets(selector, OPAQUE_PSEUDO if required_only else None)
    return {
        (sigil, name)
        for sigil, name in SELECTOR_NAME.findall(stripped)
        if not name[0].isdigit()
    }


def style_rules(rules: list[Rule]):
    for rule in rules:
        if rule.opaque:
            continue
        if not rule.at_rule:
            yield rule
        yield from style_rules(rule.children)
