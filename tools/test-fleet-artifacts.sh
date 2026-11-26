#!/usr/bin/env bash
# Deploying built artifacts: a fake `gh` serves a fixture release and fixture
# CI artifacts, so nothing is downloaded and no machine is touched. Covers the
# download checks on their own, the deployment through real Ansible, the list
# of processes left on a replaced binary, and the Mac desktop app swap
# (against doubles of the macOS tools).
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
source "$repository/tools/controller-environment.sh"
for inherited in $(compgen -e); do
    case $inherited in ANSIBLE_* | AWX_* | RUNNER_*) unset "$inherited" ;; esac
done
unset MASH_FLEET_BINARY MASH_FLEET_JOB_DIRECTORY
scratch=$(mktemp -d "${TMPDIR:?}/fleet-artifacts.XXXXXXXX")
finish() {
    local status=$? newest
    if [[ $status -ne 0 ]]; then
        # The newest job's task messages and the launcher's notes explain most failures.
        newest=$(ls -t "$scratch"/jobs/*/runner.log 2>/dev/null | head -n 1)
        [[ -z $newest ]] || grep -o '"msg": "[^"]*' "$newest" | tail -n 5 >&2
        [[ ! -f "$scratch/notes" ]] || cat "$scratch/notes" >&2
    fi
    rm -rf "$scratch"
}
trap finish EXIT
fail() { echo "$1" >&2; exit 1; }
mkdir -p "$scratch/bin" "$scratch/fake"
export FAKE_GH="$scratch/fake"
cat > "$scratch/bin/gh" <<'GH'
#!/usr/bin/env bash
# Fixture layout under $FAKE_GH:
#   release/TAG/FILE...              a published release's assets
#   runs/WORKFLOW/COMMIT             the value a successful run reports
#   artifacts/RUN/NAME/FILE...       a run's kept artifacts
set -euo pipefail
echo "$*" >> "$FAKE_GH/calls"
area=$1 action=$2
shift 2
subject='' directory='' name='' workflow='' commit='' patterns=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo | --json | --jq | --status | --limit) shift 2 ;;
        --dir) directory=$2; shift 2 ;;
        --pattern) patterns+=("$2"); shift 2 ;;
        --name) name=$2; shift 2 ;;
        --workflow) workflow=$2; shift 2 ;;
        --commit) commit=$2; shift 2 ;;
        *) subject=$1; shift ;;
    esac
done
case "$area $action" in
    'release view') ls "$FAKE_GH/release/$subject" ;;
    'release download')
        for pattern in "${patterns[@]}"; do cp "$FAKE_GH/release/$subject/$pattern" "$directory/"; done ;;
    'run list') cat "$FAKE_GH/runs/$workflow/$commit" 2>/dev/null || true ;;
    'run download') cp "$FAKE_GH/artifacts/$subject/$name"/* "$directory/" ;;
    *) echo "unexpected gh $area $action" >&2; exit 2 ;;
esac
GH
chmod 755 "$scratch/bin/gh"
export MASH_GH="$scratch/bin/gh"
export PATH="$scratch/bin:$PATH"

# Archives of stand-in programs: the right first bytes for the platform, then
# a marker the tests can recognize after installation.
archive() { # archive DIRECTORY TRIPLE MARKER
    local directory=$1 triple=$2 marker=$3 staging
    staging=$(mktemp -d "$scratch/staging.XXXXXX")
    mkdir -p "$directory"
    if [[ $triple == *windows* ]]; then
        for name in mash.exe mash-server.exe; do printf 'MZ%s %s\n' "$marker" "$name" > "$staging/$name"; done
        (cd "$staging" && python3 -m zipfile -c "$directory/mash-$triple.zip" mash.exe mash-server.exe)
    else
        for name in mash mash-server; do printf '\177ELF%s %s\n' "$marker" "$name" > "$staging/$name"; done
        tar -C "$staging" -czf "$directory/mash-$triple.tar.gz" mash mash-server
    fi
    rm -rf "$staging"
}
linux=x86_64-unknown-linux-gnu

# --- The download checks, in a checkout of their own so a tag can be made. ---
checkout="$scratch/checkout"
mkdir -p "$checkout/tools"
cp "$repository/tools/fetch-artifact.sh" "$checkout/tools/"
git -C "$checkout" init -q
git -C "$checkout" -c user.name=test -c user.email=test@example.invalid commit -q --allow-empty -m release
git -C "$checkout" tag v9.9.9
head=$(git -C "$checkout" rev-parse HEAD)
git -C "$checkout" -c user.name=test -c user.email=test@example.invalid commit -q --allow-empty -m later
git -C "$checkout" tag v9.9.10
later=$(git -C "$checkout" rev-parse HEAD)
git -C "$checkout" checkout -q --detach v9.9.9
out="$scratch/out"
platform=$(jq -nc --arg asset "$linux" --arg out "$out" '{asset:$asset,mash:($out+"/mash"),mash_server:($out+"/mash-server")}')
fetch() { bash "$checkout/tools/fetch-artifact.sh" "$@"; }
expect() { # expect STATUS COMMAND...
    local wanted=$1 status=0
    shift
    "$@" > "$scratch/fetch.log" 2>&1 || status=$?
    [[ $status == "$wanted" ]] || { cat "$scratch/fetch.log" >&2; fail "expected exit $wanted, got $status: $*"; }
}

archive "$FAKE_GH/release/v9.9.9" "$linux" released
(cd "$FAKE_GH/release/v9.9.9" && sha256sum mash-* > SHA256SUMS)
expect 0 fetch release:v9.9.9 "$platform" "$scratch/report"
grep -q 'released mash$' "$out/mash" && grep -q 'released mash-server$' "$out/mash-server"
[[ -x "$out/mash" && $(cat "$out/mash.revision") == "$head" ]]
sha256sum --check --quiet "$out/mash.sha256"
grep -q "using release v9.9.9" "$scratch/report"
# The same entry as Ansible passes it.
rm -rf "$out"
expect 0 fetch release:v9.9.9 "$(base64 -w0 <<<"$platform")"
[[ -x "$out/mash-server" ]]

# A download that does not match its checksum is refused and nothing is placed.
rm -rf "$out"
printf 'tampered' >> "$FAKE_GH/release/v9.9.9/mash-$linux.tar.gz"
expect 1 fetch release:v9.9.9 "$platform" "$scratch/report"
grep -q 'REFUSED: .*does not match its checksum' "$scratch/report"
[[ ! -e "$out/mash" && ! -e "$out/mash-server" ]]
# So is an archive the release lists no checksum for.
: > "$FAKE_GH/release/v9.9.9/SHA256SUMS"
expect 1 fetch release:v9.9.9 "$platform"
[[ ! -e "$out/mash" ]]
# A release without this platform's archive, or no release at all, means a source build.
rm "$FAKE_GH/release/v9.9.9/mash-$linux.tar.gz"
expect 3 fetch release:v9.9.9 "$platform" "$scratch/report"
grep -q "has no mash-$linux.tar.gz; building from source" "$scratch/report"
rm -rf "$FAKE_GH/release/v9.9.9"
expect 3 fetch release:v9.9.9 "$platform"
# A release that is not this checkout's commit is refused before any download.
: > "$FAKE_GH/calls"
expect 1 fetch release:v9.9.10 "$platform" "$scratch/report"
grep -q "v9.9.10 is commit ${later:0:12}" "$scratch/report"
expect 1 fetch release:v0.0.1 "$platform"
[[ ! -s "$FAKE_GH/calls" ]]

# CI's build of this commit: needs a finished artifact run and a passed CI run.
mkdir -p "$FAKE_GH/runs/dev-artifacts.yml" "$FAKE_GH/runs/ci.yml"
expect 3 fetch commit "$platform" "$scratch/report"
grep -q "no finished artifact build for ${head:0:12}" "$scratch/report"
echo 4242 > "$FAKE_GH/runs/dev-artifacts.yml/$head"
expect 3 fetch commit "$platform" "$scratch/report"
grep -q 'CI has not passed' "$scratch/report"
echo success > "$FAKE_GH/runs/ci.yml/$head"
expect 3 fetch commit "$platform" "$scratch/report"
grep -q 'kept no mash-' "$scratch/report"
kept="$FAKE_GH/artifacts/4242/mash-$linux-$head"
archive "$kept" "$linux" built
(cd "$kept" && sha256sum "mash-$linux.tar.gz" > "mash-$linux.tar.gz.sha256")
expect 0 fetch commit "$platform" "$scratch/report"
grep -q 'built mash$' "$out/mash"
grep -q "using CI build 4242 of ${head:0:12}" "$scratch/report"
rm -rf "$out"
sed -i 's/^..../0000/' "$kept/mash-$linux.tar.gz.sha256"
expect 1 fetch commit "$platform"
[[ ! -e "$out/mash" ]]
rm "$kept/mash-$linux.tar.gz.sha256"
expect 1 fetch commit "$platform"
# An archive holding another platform's programs is refused too.
rm -rf "$kept"
archive "$scratch/wrong" x86_64-pc-windows-msvc wrong
mkdir "$scratch/wrong/unzipped" "$kept"
python3 -m zipfile -e "$scratch/wrong/mash-x86_64-pc-windows-msvc.zip" "$scratch/wrong/unzipped"
mv "$scratch/wrong/unzipped/mash.exe" "$scratch/wrong/unzipped/mash"
mv "$scratch/wrong/unzipped/mash-server.exe" "$scratch/wrong/unzipped/mash-server"
tar -C "$scratch/wrong/unzipped" -czf "$kept/mash-$linux.tar.gz" mash mash-server
(cd "$kept" && sha256sum "mash-$linux.tar.gz" > "mash-$linux.tar.gz.sha256")
expect 1 fetch commit "$platform" "$scratch/report"
grep -q 'is not a x86_64-unknown-linux-gnu program' "$scratch/report"

# A Windows machine gets its programs staged over SSH with the revision file
# its installer requires.
windows=x86_64-pc-windows-msvc
kept="$FAKE_GH/artifacts/4242/mash-$windows-$head"
archive "$kept" "$windows" built
(cd "$kept" && sha256sum "mash-$windows.zip" > "mash-$windows.zip.sha256")
cat > "$scratch/bin/ssh" <<'SSH'
#!/usr/bin/env bash
echo "ssh ${*: -1}" >> "$FAKE_GH/remote"
if [[ ${*: -1} == *Get-FileHash* ]]; then
    name=$(sed -n "s/.*release\\\\\(mash[a-z-]*\.exe\)').*/\1/p" <<<"${*: -1}")
    sha256sum "$FAKE_GH/staged/$name" | cut -d ' ' -f 1
fi
SSH
cat > "$scratch/bin/scp" <<'SCP'
#!/usr/bin/env bash
mkdir -p "$FAKE_GH/staged"
echo "scp ${*: -1}" >> "$FAKE_GH/remote"
cp "${*: -2:1}" "$FAKE_GH/staged/"
SCP
chmod 755 "$scratch/bin/ssh" "$scratch/bin/scp"
expect 0 fetch commit '{"asset":"x86_64-pc-windows-msvc","windows_host":"winbox","windows_checkout":"C:\\mash-src"}' "$scratch/report"
grep -q 'built mash.exe$' "$FAKE_GH/staged/mash.exe"
grep -qF 'scp winbox:C:/mash-src/target/release/mash-server.exe' "$FAKE_GH/remote"
[[ $(tail -n 1 "$FAKE_GH/remote") == *"mash.revision' -Value '$head'"* ]]
rm "$scratch/bin/ssh" "$scratch/bin/scp"
echo 'Artifact downloads: checksum refusal, missing-asset fallback, release identity and placement passed.'

# --- The deployment, through real Ansible and the launcher. ---
revision=$(git -C "$repository" rev-parse HEAD)
cat > "$scratch/probe" <<'PROBE'
#!/usr/bin/env bash
set -euo pipefail
output=''
while [[ $# -gt 0 ]]; do
    case "$1" in --output) output=$2; shift 2 ;; *) shift ;; esac
done
printf '%s\n' '{"state":"succeeded","results":[]}' > "$output"
PROBE
chmod 755 "$scratch/probe"
built="$scratch/built" installed="$scratch/installed"
mkdir -p "$built" "$installed"
cat > "$scratch/inventory.yml" <<INVENTORY
all:
  vars:
    ansible_connection: local
    ansible_python_interpreter: $(command -v python3)
    mash_fleet: $scratch/probe
    mash_builds:
      linux:
        - [sh, -c, 'echo build >> $scratch/actions; printf compiled > $built/mash; chmod 755 $built/mash']
      other:
        - [sh, -c, 'echo other-build >> $scratch/actions']
    mash_artifacts:
      linux: {asset: $linux, mash: $built/mash, mash_server: $built/mash-server}
  children:
    mash_api:
      hosts:
        api:
          mash_platform: other
          mash_deploy_preflight: [['true']]
          mash_install: [['true']]
          mash_verify: [['true']]
    mash_history:
      hosts:
        one:
          mash_platform: linux
          mash_requires: [api]
          mash_deploy_preflight: [['true']]
          mash_install:
            - [bash, tools/install-dev-binary.sh, $built/mash, $installed/mash]
          mash_mcp: [unused]
          mash_processes:
            binaries: [$installed/mash]
          mash_desktop_install:
            - [sh, -c, 'echo "desktop {{ mash_release_tag }}" >> $scratch/actions; echo "desktop: installed"']
INVENTORY
deploy() {
    MASH_DEPLOY_INVENTORY="$scratch/inventory.yml" MASH_DEPLOY_LOGS="$scratch/jobs" \
        bash "$repository/tools/fleet.sh" deploy "$@" > "$scratch/summary.jsonl" 2> "$scratch/notes"
}
: > "$scratch/actions"
# No artifacts kept for this commit: the platform compiles, and says why.
rm -rf "$FAKE_GH/runs" "$FAKE_GH/artifacts"
deploy api one || fail 'A deployment without artifacts did not fall back to the source build.'
[[ $(grep -c '^build$' "$scratch/actions") == 1 && $(cat "$installed/mash") == compiled ]]
grep -q "Binaries: $linux: CI has no finished artifact build for ${revision:0:12}; building from source." "$scratch/notes"
grep -q 'Binaries: other: the inventory has no mash_artifacts entry' "$scratch/notes"
# CI kept this commit's build: nothing compiles for that platform, and the
# installer the inventory already names installs the download.
mkdir -p "$FAKE_GH/runs/dev-artifacts.yml" "$FAKE_GH/runs/ci.yml"
echo 77 > "$FAKE_GH/runs/dev-artifacts.yml/$revision"
echo success > "$FAKE_GH/runs/ci.yml/$revision"
kept="$FAKE_GH/artifacts/77/mash-$linux-$revision"
archive "$kept" "$linux" fetched
(cd "$kept" && sha256sum "mash-$linux.tar.gz" > "mash-$linux.tar.gz.sha256")
# A process started from the installed program before it is replaced. The
# stand-in is a copy of bash waiting on a pipe nobody writes to: it runs under
# any name (a multi-call `sleep` does not) and has no child to outlive it.
mkfifo "$scratch/hold"
cp "$(command -v bash)" "$installed/mash"
"$installed/mash" -c 'read -r -t 300 _ <> "$0"; :' "$scratch/hold" &
sleeper=$!
deploy api one || fail 'A deployment from a kept artifact failed.'
kill "$sleeper" 2>/dev/null || true
[[ $(grep -c '^build$' "$scratch/actions") == 1 ]]
grep -q 'fetched mash$' "$installed/mash"
grep -q "Binaries: $linux: using CI build 77" "$scratch/notes"
jq -se 'any(.[]; .target == "one" and .state == "succeeded")' "$scratch/summary.jsonl" >/dev/null
# Nothing supervises it as its own unit, so it is listed and left alone
# (hand-started here; on a CI runner the test itself runs inside a service).
jq -se --argjson pid "$sleeper" 'any(.[]; .target == "one" and (.stale_processes // [] | any(.pid == $pid and (.state | IN("hand-started", "unit-child", "system-unit")) and (.command | startswith("mash")))))' \
    "$scratch/summary.jsonl" >/dev/null || fail 'The process on the replaced binary was not listed.'
! grep -q '^desktop' "$scratch/actions"
# A corrupted download stops the deployment before anything is installed.
printf 'tampered' >> "$kept/mash-$linux.tar.gz"
printf previous > "$installed/mash"
if deploy api one; then fail 'A deployment accepted an artifact that failed its checksum.'; fi
[[ $(cat "$installed/mash") == previous && $(grep -c '^build$' "$scratch/actions") == 1 ]]
grep -q 'Binaries: REFUSED: .*does not match its checksum' "$scratch/notes"
jq -se 'all(.[]; .state != "succeeded" or .preflight == true)' "$scratch/summary.jsonl" >/dev/null
# --source-build never asks for an artifact.
: > "$FAKE_GH/calls"
deploy --source-build api one || fail 'A source-build deployment failed.'
[[ $(grep -c '^build$' "$scratch/actions") == 2 && ! -s "$FAKE_GH/calls" && $(cat "$installed/mash") == compiled ]]
# A release that is not this checkout's commit is refused; options need a value.
if deploy --release v0.0.0-unknown api one; then fail 'A release this checkout is not at was deployed.'; fi
grep -q 'Binaries: REFUSED: This checkout does not know the tag' "$scratch/notes"
if deploy --release api one 2>/dev/null; then fail 'A release option without a tag was accepted.'; fi
echo 'Artifact deployment: kept build installed without compiling, fallback, checksum refusal and source build passed.'

# --- The desktop step runs the machine's configured installer for a release. ---
cat > "$scratch/desktop.yml" <<YAML
- hosts: localhost
  connection: local
  gather_facts: false
  vars:
    mash_repository: "$repository"
    mash_job_directory: "$scratch"
    mash_release_tag: v9.9.9
    mash_desktop_install: [[sh, -c, 'echo "desktop: installed Mash.app {{ mash_release_tag }}"; test ! -e $scratch/desktop-fails']]
  tasks:
    - ansible.builtin.include_tasks: "$repository/deployment/playbooks/tasks/desktop.yml"
YAML
desktop_play() {
    uv run --project "$repository/deployment" --locked --no-sync ansible-playbook \
        -i localhost, "$scratch/desktop.yml" > "$scratch/desktop.log" 2>&1
}
desktop_play || { tail -n 40 "$scratch/desktop.log" >&2; fail 'The desktop step failed.'; }
grep -q 'desktop: installed Mash.app v9.9.9' "$scratch/localhost.desktop.txt"
touch "$scratch/desktop-fails"
if desktop_play; then fail 'A failed desktop installer was accepted.'; fi
grep -q 'previous app is still installed' "$scratch/localhost.desktop.txt"

# --- Processes left on a replaced binary, against a stand-in /proc and systemctl. ---
stale="$scratch/stale"
mkdir -p "$stale/proc" "$stale/bin"
printf new > "$stale/mash"
printf old > "$stale/old-mash"
printf other > "$stale/dev-build"
process() { # process PID EXECUTABLE CGROUP
    mkdir -p "$stale/proc/$1"
    ln -s "$2" "$stale/proc/$1/exe"
    printf '0::%s\n' "$3" > "$stale/proc/$1/cgroup"
}
user=/user.slice/user-1000.slice/user@1000.service
process 101 "$stale/mash (deleted)" "$user/app.slice/mash-index-watch.service"
process 102 "$stale/mash (deleted)" "$user/app.slice/agent.service"
process 103 "$stale/mash (deleted)" "/user.slice/user-1000.slice/session-3.scope"
process 104 "$stale/mash (deleted)" "/system.slice/mash-shared.service"
process 105 "$stale/mash" "$user/app.slice/mash-dash.service"
process 106 "$stale/dev-build" "$user/app.slice/mash-dash.service"
cat > "$stale/bin/systemctl" <<'SH'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_SYSTEMCTL"
case "$*" in
    '--user show mash-index-watch.service --property MainPID --value') echo 101 ;;
    '--user show agent.service --property MainPID --value') echo 1 ;;
    *) exit 0 ;;
esac
SH
chmod 755 "$stale/bin/systemctl"
export FAKE_SYSTEMCTL="$stale/systemctl.log"
list() { PATH="$stale/bin:$PATH" MASH_STALE_PROC_ROOT="$stale/proc" bash "$repository/tools/restart-stale-processes.sh" "$@" --binary "$stale/mash" | cut -f 1-4 | sort; }
[[ $(list --dry-run) == $'hand-started\t103\t-\t-\nsystem-unit\t104\tsystemd\tmash-shared.service\nunit-child\t102\tsystemd\tagent.service\nwould-restart\t101\tsystemd\tmash-index-watch.service' ]] ||
    fail 'Stale processes were not classified as expected.'
! grep -q restart "$FAKE_SYSTEMCTL"
[[ $(list | grep -c $'^restarted\t101\tsystemd\tmash-index-watch.service$') == 1 ]]
[[ $(grep -c '^--user restart' "$FAKE_SYSTEMCTL") == 1 ]] && grep -q '^--user restart mash-index-watch.service$' "$FAKE_SYSTEMCTL"
# A hand-started process the inventory names is stopped and started again.
# Only where this test is itself hand-started: inside a service (a CI runner)
# the process would belong to that unit, which is never restarted.
if [[ $(sed -n 's/^0:://p' /proc/self/cgroup) != *.service ]]; then
    cp "$(command -v bash)" "$stale/mash-live"
    "$stale/mash-live" -c 'read -r -t 300 _ <> "$0"; :' "$scratch/hold" &
    named=$!
    cp "$stale/mash-live" "$stale/mash-live.new" && mv -f "$stale/mash-live.new" "$stale/mash-live"
    result=$(bash "$repository/tools/restart-stale-processes.sh" --binary "$stale/mash-live" --restart 'mash-live -c' "touch $stale/started-again")
    [[ $(cut -f 1-3 <<<"$result") == $'restarted-by-inventory\t'"$named"$'\tinventory' ]] || fail "A named hand-started process was not restarted: $result"
    for _ in 1 2 3 4 5 6 7 8 9 10; do [[ -e "$stale/started-again" ]] && break; sleep 0.2; done
    [[ -e "$stale/started-again" ]] && ! kill -0 "$named" 2>/dev/null
fi
echo 'Stale processes: supervised units restarted once; hand-started, system and agent-owned ones only listed.'

# --- The Mac app swap, against doubles of the macOS tools. ---
mac="$scratch/mac"
mkdir -p "$mac/bin" "$mac/Applications/Mash.app/Contents" "$mac/image/Mash.app/Contents"
echo 0.2.3 > "$mac/Applications/Mash.app/Contents/version"
echo 0.2.4 > "$mac/image/Mash.app/Contents/version"
export FAKE_MAC="$mac"
double() { cat > "$mac/bin/$1"; chmod 755 "$mac/bin/$1"; }
double defaults <<'SH'
#!/usr/bin/env bash
cat "${2%/Info}/version"
SH
double hdiutil <<'SH'
#!/usr/bin/env bash
if [[ $1 == attach ]]; then
    while [[ $# -gt 0 ]]; do [[ $1 == -mountpoint ]] && mount=$2; shift; done
    cp -R "$FAKE_MAC/image/Mash.app" "$mount/"
else rm -rf "$2/Mash.app"; fi
SH
double spctl <<'SH'
#!/usr/bin/env bash
[[ "$*" == "-a -t exec -vv "* && ! -e "$FAKE_MAC/gatekeeper-rejects" ]]
SH
double codesign <<'SH'
#!/usr/bin/env bash
if [[ $1 == -dv ]]; then echo "TeamIdentifier=$(cat "$FAKE_MAC/team")" >&2; fi
SH
double osascript <<'SH'
#!/usr/bin/env bash
echo quit >> "$FAKE_MAC/actions"; rm -f "$FAKE_MAC/running"
SH
double pkill <<'SH'
#!/usr/bin/env bash
rm -f "$FAKE_MAC/running"
SH
double pgrep <<'SH'
#!/usr/bin/env bash
[[ -e "$FAKE_MAC/running" ]]
SH
double ditto <<'SH'
#!/usr/bin/env bash
cp -R "$1" "$2"
SH
double open <<'SH'
#!/usr/bin/env bash
echo "open $(cat "$1/Contents/version")" >> "$FAKE_MAC/actions"
[[ -e "$FAKE_MAC/new-app-crashes" && $(cat "$1/Contents/version") == 0.2.4 ]] || touch "$FAKE_MAC/running"
SH
double sleep <<'SH'
#!/usr/bin/env bash
exit 0
SH
swap() { PATH="$mac/bin:$PATH" MASH_DESKTOP_APPLICATIONS="$mac/Applications" bash "$repository/tools/desktop-macos-swap.sh" "$mac/image.dmg" 0.2.4 > "$mac/out" 2>&1; }
installed_version() { cat "$mac/Applications/Mash.app/Contents/version"; }
echo JZ665W692Z > "$mac/team"
# Another team's app, or one Gatekeeper rejects, changes nothing.
echo OTHERTEAM1 > "$mac/team"
if swap; then fail 'An app signed by another team was installed.'; fi
[[ $(installed_version) == 0.2.3 ]] && grep -q 'not signed by team JZ665W692Z' "$mac/out"
echo JZ665W692Z > "$mac/team"
touch "$mac/gatekeeper-rejects"
if swap; then fail 'An app Gatekeeper rejects was installed.'; fi
[[ $(installed_version) == 0.2.3 ]]
rm "$mac/gatekeeper-rejects"
# A new app that does not start is rolled back, and the old one reopened.
touch "$mac/running" "$mac/new-app-crashes"
if swap; then fail 'An app that did not start was kept.'; fi
[[ $(installed_version) == 0.2.3 && ! -e "$mac/Applications/Mash.app.previous" ]]
[[ $(tail -n 1 "$mac/actions") == 'open 0.2.3' && -e "$mac/running" ]]
rm "$mac/new-app-crashes"
# The running app is quit, replaced and reopened; the previous copy is removed.
: > "$mac/actions"
swap || { cat "$mac/out" >&2; fail 'The app swap failed.'; }
[[ $(installed_version) == 0.2.4 && ! -e "$mac/Applications/Mash.app.previous" && -e "$mac/running" ]]
[[ $(cat "$mac/actions") == $'quit\nopen 0.2.4' ]]
grep -q 'installed Mash.app 0.2.4 and reopened it' "$mac/out"
# Already at this version: nothing is quit or opened.
: > "$mac/actions"
swap && [[ ! -s "$mac/actions" ]] && grep -q 'already installed' "$mac/out"
# An app that was not open is closed again after it proves it starts.
echo 0.2.3 > "$mac/Applications/Mash.app/Contents/version"
rm "$mac/running"
swap || { cat "$mac/out" >&2; fail 'The app swap failed for a closed app.'; }
[[ $(installed_version) == 0.2.4 && ! -e "$mac/running" && $(cat "$mac/actions") == $'open 0.2.4\nquit' ]]
echo 'Desktop app swap: signature and team checks, rollback when the new app does not start, reopen passed (macOS tool doubles).'
