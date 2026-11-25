#!/bin/bash
# Runs inside a fresh clone of the golden image for `tools/mac-vm.sh run`:
# installs the Mash build under test, indexes the made-up demo histories,
# records the demo decisions, and starts the Raycast extension in development
# mode. Everything it writes lives in this disposable clone.
#
# Usage: guest-run.sh   (expects ~/mac-vm/mash/ with the mash binaries,
#                        ~/mac-vm/raycast/, and optionally ~/mac-vm/models/)
set -euo pipefail
eval "$(/opt/homebrew/bin/brew shellenv)"
export PATH="/opt/homebrew/opt/node@22/bin:$HOME/.local/bin:$PATH"
SRC=$HOME/mac-vm

# Also set by guest-image.sh; repeated for images built before it was.
defaults write com.apple.WindowManager EnableStandardClickToShowDesktop -bool false
killall WindowManager 2>/dev/null || true

mkdir -p ~/.local/bin
install -m 755 "$SRC"/mash/mash* ~/.local/bin/
mash --version

# Fresh timestamps, so the demo chats read as today and this week.
"$SRC/tools/demo_history.py" "$HOME"

if [ -d "$SRC/models" ]; then
  mkdir -p ~/.mash && rm -rf ~/.mash/models && cp -R "$SRC/models" ~/.mash/models
fi
mash qdrant install >/dev/null
mash models install >/dev/null
# Agents stay unconnected: init only finds their histories and indexes them.
mash init --yes --no-agents >/dev/null 2>&1
mash index >/dev/null 2>&1
"$SRC/tools/demo_records.py" mash

cd "$SRC/raycast"
npm ci --no-audit --no-fund >/dev/null
nohup npm run dev > ~/raycast-dev.log 2>&1 &
for _ in $(seq 1 90); do
  grep -q "built extension successfully" ~/raycast-dev.log && break
  sleep 1
done
grep -q "built extension successfully" ~/raycast-dev.log || { cat ~/raycast-dev.log >&2; exit 1; }

# Semantic search answers once Qdrant has the collection; wait for it so the
# first screenshot is not of a warning.
for _ in $(seq 1 60); do
  curl -sf http://127.0.0.1:6333/collections | grep -q '"name"' && break
  sleep 2
done
mash search "slow card payments" --limit 1 | head -c 160
echo
