import { projects } from "./history";

interface DemoRecord {
  readonly id: string;
  readonly kind: "decision" | "preference";
  readonly statement: string;
  readonly rationale: string;
  readonly review: "candidate" | "established";
  readonly recordedBy: string;
  readonly project: string | null;
  readonly daysAgo: number;
  readonly alternatives?: readonly string[];
  readonly consequences?: readonly string[];
  /** The chat and the position of the message the record quotes. */
  readonly source: { readonly chat: string; readonly message: number; readonly quote: string };
}

export const records: readonly DemoRecord[] = [
  {
    id: "record-sign-in-tokens",
    kind: "decision",
    statement:
      "Sign-in uses fifteen-minute access tokens with a server-held refresh token. The mobile app follows the same scheme.",
    rationale: "Chosen so that signing out reaches every device within fifteen minutes.",
    review: "established",
    recordedBy: "Claude Code",
    project: projects.storefront.repository,
    daysAgo: 0.1,
    alternatives: ["Keep session cookies and add a server-side session table"],
    consequences: ["The mobile app stores its refresh token in the device keychain"],
    source: {
      chat: "sign-in-tokens",
      message: 2,
      quote: "ok, go with the tokens. keep the mobile app on the same scheme",
    },
  },
  {
    id: "record-export-queue",
    kind: "decision",
    statement: "Invoice exports run as background jobs kept in a Postgres table.",
    rationale:
      "A job is created in the same transaction as the export request, so it cannot be lost.",
    review: "established",
    recordedBy: "Claude Code",
    project: projects.billing.repository,
    daysAgo: 2.3,
    alternatives: ["Redis streams", "A hosted queue"],
    consequences: [
      "Workers claim a job for ten minutes at a time and extend the claim while working",
    ],
    source: { chat: "invoice-export-queue", message: 2, quote: "agreed, postgres." },
  },
  {
    id: "record-orders-lock",
    kind: "preference",
    statement: "Never take a write lock on the orders table in a migration.",
    rationale: "A locking index build stopped checkout for forty minutes.",
    review: "established",
    recordedBy: "Codex",
    project: projects.infra.repository,
    daysAgo: 41.3,
    source: {
      chat: "checkout-outage-review",
      message: 4,
      quote: "remember that we never lock the orders table in a migration",
    },
  },
  {
    id: "record-refund-cents",
    kind: "decision",
    statement: "Refunds are calculated in whole cents, and the last part takes the remainder.",
    rationale: "Rounding each part on its own refunded more than the order total.",
    review: "candidate",
    recordedBy: "GitHub Copilot",
    project: projects.billing.repository,
    daysAgo: 9.3,
    source: { chat: "refund-rounding", message: 2, quote: "yes. and add a test for it" },
  },
  {
    id: "record-key-rotation",
    kind: "decision",
    statement:
      "Deploy keys are rotated every ninety days, and whenever someone with access leaves.",
    rationale: "The keys had not been changed since the project started.",
    review: "established",
    recordedBy: "Claude Code",
    project: projects.infra.repository,
    daysAgo: 15.6,
    alternatives: [],
    consequences: ["A scheduled job opens a reminder issue every ninety days"],
    source: { chat: "rotate-deploy-keys", message: 2, quote: "go ahead. ninety days is fine" },
  },
  {
    id: "record-brand-orange",
    kind: "preference",
    statement: "The brand orange stays the same in the light and dark themes.",
    rationale: "Asked for when the dark theme was added.",
    review: "candidate",
    recordedBy: "OpenCode",
    project: projects.storefront.repository,
    daysAgo: 27.1,
    source: {
      chat: "dark-theme",
      message: 2,
      quote: "keep the brand orange the same in both",
    },
  },
];
