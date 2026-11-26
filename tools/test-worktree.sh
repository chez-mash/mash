#!/usr/bin/env bash
# Checks tools/worktree.sh against a throwaway bare remote and clone: new,
# land (rebased, checked, fast-forward only), done and its refusals, and the
# deploy tree. Nothing here touches the network or the real repository.
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/worktree-test.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
export GIT_AUTHOR_NAME=Fixture GIT_AUTHOR_EMAIL=fixture@example.invalid
export GIT_COMMITTER_NAME=Fixture GIT_COMMITTER_EMAIL=fixture@example.invalid
# The check must leave the tree clean without help from the environment.
unset MASH_WORKTREES PYTHONDONTWRITEBYTECODE PYTHONPYCACHEPREFIX
fail() {
    printf 'FAIL %s\n' "$1" >&2
    [[ ! -f $scratch/output ]] || sed 's/^/  /' "$scratch/output" >&2
    exit 1
}
# refused NAME PATTERN COMMAND...: the command fails and says why.
refused() {
    local name=$1 pattern=$2
    shift 2
    if "$@" > "$scratch/output" 2>&1; then fail "$name: was not refused"; fi
    grep -Eq "$pattern" "$scratch/output" || fail "$name: no message matching $pattern"
}
remote_main() { git -C "$scratch/remote.git" rev-parse main; }

git init -q --bare -b main "$scratch/remote.git"
main=$scratch/mash
git init -q -b main "$main"
mkdir -p "$main/tools" "$main/.githooks" "$main/src"
cp "$repository/tools/worktree.sh" "$repository/tools/pre-push.sh" "$repository/tools/install-hooks.sh" \
    "$repository/tools/check_file_lines.py" "$repository/tools/file_growth.py" "$main/tools/"
cp "$repository/tools/worktree-preview.sh" "$main/tools/"
cp "$repository/.githooks/pre-push" "$main/.githooks/"
: > "$main/tools/file-lines-baseline.txt"
printf 'one\n' > "$main/notes.md"
# Ignored in the real repository: the agent skills and the Claude settings.
mkdir -p "$main/.agents/skills/example" "$main/.agents/plugins" "$main/.claude"
printf 'skill\n' > "$main/.agents/skills/example/SKILL.md"
printf '{}\n' > "$main/.agents/plugins/marketplace.json"
printf '{}\n' > "$main/.claude/settings.json"
printf '/.agents/*\n!/.agents/plugins/\n/.claude/\n' > "$main/.gitignore"
git -C "$main" add -A
git -C "$main" commit -qm fixture
git -C "$main" remote add origin "$scratch/remote.git"
git -C "$main" push -q origin main > /dev/null 2>&1
bash "$main/tools/install-hooks.sh" > /dev/null
worktree() { bash "$main/tools/worktree.sh" "$@"; }

refused 'a name with a slash' 'a name starts with' worktree new bad/name
tree=$(worktree new alpha 2> "$scratch/output") || fail 'new alpha'
[[ $tree == "$scratch/mash-worktrees/alpha" && -d $tree ]] || fail "new prints the worktree path, got $tree"
[[ $(git -C "$tree" rev-parse --abbrev-ref HEAD) == work/alpha ]] || fail 'new creates branch work/alpha'
[[ $(git -C "$tree" rev-parse HEAD) == $(remote_main) ]] || fail 'new starts from origin/main'
grep -q DEV_TARGET_DIR "$scratch/output" || fail 'new explains the shared Cargo target'
[[ -L $tree/.agents/skills && -f $tree/.agents/skills/example/SKILL.md ]] || fail 'new links the skills from the main checkout'
[[ -L $tree/.claude && -f $tree/.claude/settings.json ]] || fail 'new links .claude from the main checkout'
[[ -z $(git -C "$tree" status --porcelain) ]] || fail 'the links leave the new worktree clean'
grep -q 'bd ready' "$scratch/output" || fail 'new says where to run bd'
refused 'a second worktree of the same name' 'already exists' worktree new alpha
worktree list | grep -q 'work/alpha' || fail 'list shows the worktree'

# Main moves while alpha is in progress, as another session's landing would.
printf 'two\n' > "$main/other.md"
git -C "$main" add other.md
git -C "$main" commit -qm other
git -C "$main" push -q origin main > /dev/null 2>&1

printf 'alpha\n' > "$tree/alpha.md"
refused 'done with an untracked file' 'uncommitted or untracked' worktree done alpha
git -C "$tree" add alpha.md
refused 'land with staged changes' 'uncommitted changes' worktree land alpha
git -C "$tree" commit -qm alpha
refused 'done with unlanded commits' 'not on origin/main' worktree done alpha

# A file over the line limit stops the landing before anything is pushed.
mkdir -p "$tree/src"
seq 1 501 | sed 's/^/const LINE_&: u16 = &;/' > "$tree/src/long.rs"
git -C "$tree" add src/long.rs
git -C "$tree" commit -qm long
before=$(remote_main)
refused 'land with a failing check' 'pre-push check failed' worktree land alpha
[[ $(remote_main) == "$before" ]] || fail 'a failed landing reached the remote'
git -C "$tree" reset -q --hard HEAD~1

worktree land alpha > "$scratch/output" 2>&1 || fail 'land alpha'
[[ $(remote_main) == $(git -C "$tree" rev-parse HEAD) ]] || fail 'land fast-forwards origin main to the branch'
git -C "$scratch/remote.git" cat-file -e main:other.md || fail 'land rebased onto the moved main'
[[ $(git -C "$scratch/remote.git" rev-list --count --merges main) == 0 ]] || fail 'land keeps history linear'
[[ -d $tree ]] || fail 'land leaves the worktree for done'
worktree done alpha > "$scratch/output" 2>&1 || fail 'done alpha'
[[ ! -e $tree ]] || fail 'done removes the worktree'
! git -C "$main" show-ref --verify --quiet refs/heads/work/alpha || fail 'done deletes the merged branch'

# --force discards a dirty, unlanded worktree.
tree=$(worktree new beta 2> /dev/null)
printf 'beta\n' > "$tree/beta.md"
git -C "$tree" add beta.md
git -C "$tree" commit -qm beta
printf 'draft\n' > "$tree/draft.md"
refused 'done on unfinished work' 'uncommitted or untracked' worktree done beta
worktree done beta --force > "$scratch/output" 2>&1 || fail 'done --force'
[[ ! -e $tree ]] || fail 'done --force removes the worktree'
! git -C "$main" show-ref --verify --quiet refs/heads/work/beta || fail 'done --force deletes the branch'

# The deploy tree follows origin/main, detached, and refuses local changes.
deploy=$(worktree deploy-tree 2> /dev/null)
[[ $deploy == "$scratch/mash-worktrees/_deploy" ]] || fail "deploy-tree prints its path, got $deploy"
[[ $(git -C "$deploy" rev-parse HEAD) == $(remote_main) ]] || fail 'the deploy tree is at origin/main'
[[ $(git -C "$deploy" rev-parse --abbrev-ref HEAD) == HEAD ]] || fail 'the deploy tree is detached'
git -C "$main" pull -q --ff-only origin main
printf 'three\n' > "$main/third.md"
git -C "$main" add third.md
git -C "$main" commit -qm third
git -C "$main" push -q origin main > /dev/null 2>&1
worktree deploy-tree > /dev/null 2>&1 || fail 'deploy-tree refreshes'
[[ $(git -C "$deploy" rev-parse HEAD) == $(remote_main) ]] || fail 'the deploy tree follows origin/main'
printf 'stray\n' > "$deploy/stray.md"
refused 'a deploy tree with local changes' 'must stay clean' worktree deploy-tree

# Previews: stand-ins for Tailscale, systemd, pnpm, ss and curl record what a
# preview would start, so no server runs and nothing leaves this directory.
fakes=$scratch/fakes
mkdir -p "$fakes" "$scratch/units" "$scratch/runs"
cat > "$fakes/tailscale" << 'FAKE'
#!/usr/bin/env bash
[[ ! -e $FAKE_ROOT/tailscale-down ]] || exit 1
case "$*" in
    'ip -4') echo "${FAKE_TAILNET_ADDRESS:-100.101.102.103}" ;;
    'status --json') echo '{"Self": {"DNSName": "fixture.example.ts.net."}}' ;;
esac
FAKE
cat > "$fakes/systemd-run" << 'FAKE'
#!/usr/bin/env bash
arguments=$*
while [[ $# -gt 0 && $1 != --unit ]]; do shift; done
printf '%s\n' "$arguments" > "$FAKE_ROOT/runs/$2"
touch "$FAKE_ROOT/units/$2.service"
FAKE
cat > "$fakes/systemctl" << 'FAKE'
#!/usr/bin/env bash
case "$*" in
    '--user is-active --quiet '*) [[ -e $FAKE_ROOT/units/$4 ]] ;;
    '--user stop '*) rm -f "$FAKE_ROOT/units/$3" ;;
    *) exit 0 ;;
esac
FAKE
cat > "$fakes/pnpm" << 'FAKE'
#!/usr/bin/env bash
printf '%s %s\n' "$(basename "$PWD")" "$*" >> "$FAKE_ROOT/pnpm"
[[ $1 != install ]] || mkdir -p node_modules
FAKE
cat > "$fakes/ss" << 'FAKE'
#!/usr/bin/env bash
last=${!#}
port=${last##*:}
! grep -qx "$port" "$FAKE_ROOT/busy-ports" 2> /dev/null || echo "LISTEN 0 511 0.0.0.0:$port 0.0.0.0:*"
FAKE
printf '#!/usr/bin/env bash\nexit 0\n' > "$fakes/curl"
chmod +x "$fakes"/*
export FAKE_ROOT=$scratch MASH_PREVIEW_STATE=$scratch/previews
unset MASH_DASHBOARD_API_URL MASH_PREVIEW_MEMORY_MAX XDG_STATE_HOME
preview() { PATH=$fakes:$PATH bash "$main/tools/worktree.sh" "$@"; }
printf 'node_modules/\n' >> "$main/.git/info/exclude"

tree=$(worktree new gamma 2> /dev/null)
mkdir -p "$tree/dashboard" "$tree/site"
# cksum of the name, modulo the hundred ports of the range: fixed for a name.
gamma_port=5242
touch "$scratch/tailscale-down"
refused 'a preview without Tailscale' 'Tailscale is not up' preview preview gamma
[[ -z $(ls "$scratch/runs") ]] || fail 'a refused preview started a unit'
rm "$scratch/tailscale-down"
FAKE_TAILNET_ADDRESS=203.0.113.9 refused 'a preview on an address outside the tailnet range' 'Tailscale is not up' preview preview gamma
refused 'a preview of a missing worktree' 'no worktree at' preview preview missing
refused 'a Storybook preview' 'Storybook is not set up yet' preview preview gamma --storybook
refused 'real history on the website' 'dashboard only' preview preview gamma --site --real

preview preview gamma > "$scratch/addresses" 2> "$scratch/output" || fail 'preview gamma'
run=$(cat "$scratch/runs/mash-preview-gamma")
[[ $(sed -n 1p "$scratch/addresses") == "http://100.101.102.103:$gamma_port/demo/" ]] || fail "preview prints the tailnet address, got $(cat "$scratch/addresses")"
[[ $(sed -n 2p "$scratch/addresses") == "http://fixture.example.ts.net:$gamma_port/demo/" ]] || fail 'preview prints the MagicDNS address'
[[ $run == *"run dev:demo --host 100.101.102.103 --port $gamma_port --strictPort" ]] || fail "the demo dev server binds the tailnet address, got $run"
[[ $run != *0.0.0.0* && $run != *'--host 127.'* ]] || fail 'a preview listens beyond the tailnet address'
[[ $run == *'MemoryMax=1500M'* ]] || fail 'a preview has a memory cap'
[[ $run == *'__VITE_ADDITIONAL_SERVER_ALLOWED_HOSTS=fixture.example.ts.net'* ]] || fail 'Vite is told the MagicDNS name'
[[ $run == *"WorkingDirectory=$tree/dashboard"* ]] || fail 'the dev server runs in the worktree'
[[ $run != *MASH_DASHBOARD_API_URL* && $run != *EnvironmentFile* ]] || fail 'a demo preview reaches for the real dashboard'
grep -qx 'dashboard install --frozen-lockfile --prefer-offline' "$scratch/pnpm" || fail 'preview installs missing dependencies'
! grep -q 'REAL HISTORY' "$scratch/output" || fail 'a demo preview warns about real history'

# The same name keeps its port: asked again while running, and after a stop.
preview preview gamma > "$scratch/again" 2> "$scratch/output" || fail 'preview gamma again'
grep -q 'already running' "$scratch/output" || fail 'a second start says the preview is running'
cmp -s "$scratch/addresses" "$scratch/again" || fail 'a second start prints the same addresses'
[[ $(grep -c install "$scratch/pnpm") == 1 ]] || fail 'dependencies are installed once'
preview preview gamma --stop > "$scratch/output" 2>&1 || fail 'preview --stop'
[[ ! -e $scratch/units/mash-preview-gamma.service && ! -e $scratch/previews/mash-preview-gamma.state ]] || fail '--stop leaves the unit or its record'
preview preview gamma > "$scratch/addresses" 2> /dev/null
grep -q ":$gamma_port/" "$scratch/addresses" || fail 'the port is derived from the name alone'
preview preview gamma --stop > /dev/null 2>&1

# A busy port moves the preview to the next free one, which it then keeps.
echo "$gamma_port" > "$scratch/busy-ports"
preview preview gamma > "$scratch/addresses" 2> /dev/null
grep -q ":$((gamma_port + 1))/" "$scratch/addresses" || fail 'a busy port is skipped'
rm "$scratch/busy-ports"
preview preview gamma --stop > /dev/null 2>&1

# Real history needs the flag, says who can read it, and never shows a token.
printf 'MASH_DASH_TOKEN=fixture-secret\n' > "$scratch/dashboard.env"
MASH_PREVIEW_DASH_ENV=$scratch/dashboard.env preview preview gamma --real > "$scratch/addresses" 2> "$scratch/output" || fail 'preview --real'
run=$(cat "$scratch/runs/mash-preview-gamma")
grep -q 'REAL HISTORY' "$scratch/output" && grep -q 'Anyone on the tailnet' "$scratch/output" || fail '--real says who can read the history'
[[ $run == *'MASH_PREVIEW_READ_ONLY=1'* && $run == *'MASH_DASHBOARD_API_URL=http://127.0.0.1:8790'* ]] || fail "--real proxies read-only, got $run"
[[ $run == *"EnvironmentFile=$scratch/dashboard.env"* && $run == *'run dev --host 100.101.102.103'* ]] || fail '--real hands the token file to the dev server'
! grep -rq fixture-secret "$scratch/output" "$scratch/addresses" "$scratch/runs" "$scratch/previews" || fail 'the token was printed or recorded'
[[ $(cat "$scratch/addresses") == *":$gamma_port/"* && $(cat "$scratch/addresses") != */demo/* ]] || fail '--real serves the dashboard at the root'

# The website gets its own unit and range; --local stays on this machine.
preview preview gamma --site --local > "$scratch/addresses" 2> "$scratch/output" || fail 'preview --site --local'
[[ $(cat "$scratch/addresses") == "http://127.0.0.1:$((gamma_port + 100))/" ]] || fail "--site --local prints one loopback address, got $(cat "$scratch/addresses")"
[[ $(cat "$scratch/runs/mash-preview-gamma-site") == *"WorkingDirectory=$tree/site"*"run dev --host 127.0.0.1 --port $((gamma_port + 100))" ]] || fail 'the website preview runs astro dev on loopback'
preview preview gamma --site --stop > /dev/null 2>&1
preview preview gamma --site 2> /dev/null > "$scratch/addresses" || fail 'preview --site'
[[ $(cat "$scratch/runs/mash-preview-gamma-site") == *'--allowed-hosts fixture.example.ts.net' ]] || fail 'Astro is told the MagicDNS name'

preview previews > "$scratch/list" 2> "$scratch/output" || fail 'previews'
grep -Eq "^gamma +dashboard +real +[0-9]+s +answers +http://fixture.example.ts.net:$gamma_port/\$" "$scratch/list" || fail "previews lists the dashboard preview, got $(cat "$scratch/list")"
grep -Eq "^gamma +site +docs +[0-9]+s +answers +http://fixture.example.ts.net:$((gamma_port + 100))/\$" "$scratch/list" || fail 'previews lists the website preview'
rm "$scratch/units/mash-preview-gamma-site.service"
preview previews > "$scratch/list" 2> /dev/null
grep -Eq '^gamma +site .* stopped ' "$scratch/list" || fail 'previews shows a preview that died as stopped'
touch "$scratch/units/mash-preview-gamma-site.service"

# A worktree whose name would reuse another preview's unit name is refused.
other=$(worktree new gamma-site 2> /dev/null)
mkdir -p "$other/dashboard"
refused 'a unit name that belongs to another preview' 'belongs to the site preview of gamma' preview preview gamma-site
worktree done gamma-site > /dev/null 2>&1 || fail 'done gamma-site'
[[ -e $scratch/units/mash-preview-gamma-site.service ]] || fail 'done stopped the preview of another worktree'

# done stops every preview of its worktree before removing it.
preview done gamma > "$scratch/output" 2>&1 || fail 'done gamma with previews'
[[ -z $(ls "$scratch/units") && -z $(ls "$scratch/previews"/*.state 2> /dev/null) ]] || fail 'done leaves a preview running or recorded'
[[ ! -e $tree ]] || fail 'done removes a worktree that had previews'
preview previews > "$scratch/list" 2>&1
grep -q 'No previews' "$scratch/list" || fail 'previews says when there are none'
rm -f "$scratch/output"
echo 'worktree: ok'
