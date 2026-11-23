// A one-page site for `pnpm test:review`, served from a fresh copy in
// tests/review/.fixture/ so the tests can edit its files. Nothing under
// site/src is touched, so a running `make site-review` never reloads.
import { defineConfig } from "astro/config";
import { agentationReview } from "../../../src/agentation.ts";

export default defineConfig({ integrations: [agentationReview()] });
