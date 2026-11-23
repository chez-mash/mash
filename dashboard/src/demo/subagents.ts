// Demo chats that started subagents, and the subagents themselves, for both
// Claude Code and Codex. Made up, like the rest of the demo's history.
import type { DemoChat, DemoMessage } from "./history";

/** What an agent recorded about a subagent chat it started. */
export interface DemoSubagent {
  readonly parent: string;
  readonly label: string;
  readonly role: string;
  readonly finished?: boolean;
  /** The position of the parent's tool call that started it. */
  readonly spawnedAt?: number;
}

interface Place {
  readonly path: string;
  readonly repository: string;
}

const user = (content: string): DemoMessage => ({ role: "user", content });
const agent = (content: string): DemoMessage => ({ role: "assistant", content });
const spawn = (description: string, prompt: string): DemoMessage => ({
  role: "assistant",
  content: "",
  call: { name: "Agent", input: { description, prompt, subagent_type: "general-purpose" } },
});
const spawned = (output: string, call: number): DemoMessage => ({
  role: "tool",
  content: output,
  answers: call,
});

function base(
  id: string,
  title: string,
  by: DemoChat["agent"],
  project: Place,
  daysAgo: number,
  messages: readonly DemoMessage[],
): DemoChat {
  return {
    id,
    title,
    agent: by,
    machine: by === "codex" ? "workstation" : "laptop",
    project: project.path,
    repository: project.repository,
    daysAgo,
    messages,
  };
}

/** The demo's subagent family, in the project folders the demo uses. */
export function subagentChats(storefront: Place, mobile: Place): DemoChat[] {
  return [
    base("release-0-2", "Ship the 0.2.0 release", "claude-code", storefront, 0.008, [
      user(
        "the windows tests have been failing since friday, can you get them green and cut 0.2.0?",
      ),
      agent(
        "I'll hand the Windows failures to a subagent that can reproduce them, and write the release notes in another while I prepare the tag here.",
      ),
      spawn(
        "Fix the failing Windows test suite",
        "The Windows workflow has failed on every main commit since Friday. Find which commits broke each test and fix the real cause.",
      ),
      spawned("Three tests asserted wording changed on Friday; fixed in two commits.", 2),
      spawn(
        "Draft the 0.2.0 release notes",
        "Summarise the changes since 0.1.4 for the release page.",
      ),
      spawned("Release notes drafted in CHANGELOG.md.", 4),
      agent(
        "The subagent found three tests still asserting old wording. With those fixed, CI and the Windows run both pass, so I've tagged `v0.2.0`.",
      ),
    ]),
    {
      ...base(
        "release-windows-tests",
        "Work ONLY in /home/sam/code/storefront",
        "claude-code",
        storefront,
        0.04,
        [
          user(
            "Work ONLY in /home/sam/code/storefront. The Windows workflow has failed on every main commit since Friday. Find which commits broke each test and fix the real cause.",
          ),
          agent(
            "None of the failures were Windows-specific: three tests still asserted wording changed on Friday. Fixed in two commits; the Windows workflow now passes.",
          ),
        ],
      ),
      subagent: {
        parent: "release-0-2",
        label: "Fix the failing Windows test suite",
        role: "general-purpose",
        finished: true,
        spawnedAt: 2,
      },
    },
    {
      ...base(
        "release-notes",
        "Summarise the changes since 0.1.4",
        "claude-code",
        storefront,
        0.02,
        [
          user("Summarise the changes since 0.1.4 for the release page."),
          agent(
            "Release notes drafted in CHANGELOG.md under 0.2.0: token sign-in, faster search, and the Windows fixes.",
          ),
        ],
      ),
      subagent: {
        parent: "release-0-2",
        label: "Draft the 0.2.0 release notes",
        role: "general-purpose",
        spawnedAt: 4,
      },
    },
    base("docs-hierarchy", "Tidy the docs hierarchy", "codex", mobile, 0.05, [
      user("Merge the duplicate install pages and give every guide one name."),
      agent(
        "I've asked two subagents to survey the guides and check the links while I merge the install pages.",
      ),
      agent(
        "The install pages are merged; every guide now has one name, and the old addresses redirect.",
      ),
    ]),
    {
      ...base("docs-survey", "Survey every guide", "codex", mobile, 0.06, [
        user("List every guide, its title and its address, and mark duplicates."),
        agent(
          "Two install pages cover the same steps; the Windows installer page links to an old test build.",
        ),
      ]),
      subagent: { parent: "docs-hierarchy", label: "Popper", role: "explorer", finished: true },
    },
    {
      ...base("docs-links", "Check the docs links", "codex", mobile, 0.055, [
        user("Check every link in the guides and report the broken ones."),
        agent("Four links were broken; all now point at the merged install page."),
      ]),
      subagent: { parent: "docs-hierarchy", label: "Locke", role: "worker", finished: true },
    },
  ];
}
