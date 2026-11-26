#!/usr/bin/env bash
# Point Git at the tracked hooks in .githooks/. The setting lives in the
# repository's shared configuration, so one run covers every worktree of this
# clone; run it once after cloning.
set -euo pipefail
cd "$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
git config core.hooksPath .githooks
echo "Git hooks: $(git config core.hooksPath) (pre-push runs tools/pre-push.sh)"
