#!/usr/bin/env bash
# Invoke the existing Ansible policy directly; retain private reports for review.
set -euo pipefail
usage() {
    echo 'usage: fleet.sh OPERATION TARGET...|--all' >&2
    echo '       fleet.sh deploy [--release vX.Y.Z | --source-build] TARGET...|--all' >&2
    exit 2
}
[[ $# -ge 2 ]] || usage
operation=$1
shift
# A deployment installs the binaries CI built for this commit when it kept
# them, and compiles otherwise. --release installs a published release's
# assets (and its desktop app where the inventory asks); --source-build
# always compiles.
artifact_source=''
release_tag=''
if [[ $operation == deploy ]]; then
    artifact_source=commit
    case "$1" in
        --release)
            [[ $# -ge 3 && "$2" =~ ^v[0-9][0-9A-Za-z.+-]*$ ]] || usage
            release_tag=$2 artifact_source="release:$2"
            shift 2 ;;
        --source-build) artifact_source=source; shift ;;
    esac
    [[ $# -ge 1 ]] || usage
fi
case "$operation" in
    preflight|deploy|check|doctor) playbook=$operation ;;
    sync|local-views) playbook=refresh ;;
    replay-views) playbook=replay-views ;;
    watch-start|watch-stop|watch-status|watch-logs) playbook=watch ;;
    *) echo 'Unknown fleet operation.' >&2; exit 2 ;;
esac
for target in "$@"; do
    [[ $# == 1 && "$target" == --all ]] && continue
    [[ "$target" =~ ^[A-Za-z0-9][A-Za-z0-9_.-]*$ ]] || { echo 'Invalid target name.' >&2; exit 2; }
done
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
inventory=${MASH_DEPLOY_INVENTORY:-$HOME/.config/mash/dev-inventory.yml}
logs=${MASH_DEPLOY_LOGS:-/home/james/scratch/mash-artifacts/dev-jobs}
[[ -f "$inventory" ]] || { echo 'Set MASH_DEPLOY_INVENTORY to a private inventory.' >&2; exit 2; }
umask 077
mkdir -p "$logs"
if [[ -n ${MASH_FLEET_JOB_DIRECTORY:-} ]]; then
    job=$MASH_FLEET_JOB_DIRECTORY
    mkdir -m 700 "$job"
else
    job=$(mktemp -d "$logs/$operation.XXXXXXXX")
fi
mkdir "$job/env"
export ANSIBLE_CONFIG="$repository/deployment/ansible.cfg"
source "$repository/tools/controller-environment.sh"
if [[ $# == 1 && "$1" == --all ]]; then
    uv run --project "$repository/deployment" --locked --no-sync \
        ansible-inventory --inventory "$inventory" --list > "$job/inventory.json"
    case "$operation" in
        sync|local-views|watch-*) selected_groups='["mash_history"]' ;;
        replay-views) selected_groups='["mash_api"]' ;;
        *) selected_groups='["mash_api","mash_history"]' ;;
    esac
    targets=()
    while IFS= read -r target; do targets+=("$target"); done < <(jq -r --argjson groups "$selected_groups" '
        . as $inventory |
        def hosts: . as $name | ($inventory[$name].hosts // [])[],
            (($inventory[$name].children // [])[] | hosts);
        [$groups[] | hosts] | unique[]' "$job/inventory.json")
    [[ ${#targets[@]} -gt 0 ]] || { echo 'No eligible inventory hosts.' >&2; exit 2; }
    set -- "${targets[@]}"
    for target in "$@"; do
        [[ "$target" =~ ^[A-Za-z0-9][A-Za-z0-9_.-]*$ ]] || { echo 'Invalid inventory host name.' >&2; exit 2; }
    done
fi
jq -n --arg operation "$operation" --arg repository "$repository" --arg job "$job" \
    --arg fleet "${MASH_FLEET_BINARY:-}" --arg artifact_source "$artifact_source" --arg release_tag "$release_tag" \
    --args '{mash_operation:$operation,mash_repository:$repository,mash_job_directory:$job,mash_selected_nodes:$ARGS.positional}
        + (if $fleet == "" then {} else {mash_fleet:$fleet} end)
        + (if $artifact_source == "" then {} else {mash_artifact_source:$artifact_source} end)
        + (if $release_tag == "" then {} else {mash_release_tag:$release_tag} end)' \
    "$@" > "$job/env/extravars"
result=0
bash "$repository/tools/run-ansible.sh" "$logs/fleet.lock" \
    timeout 7200 uv run --project "$repository/deployment" --locked --no-sync \
    ansible-runner run "$job" --project-dir "$repository" --inventory "$inventory" \
    --playbook "deployment/playbooks/$playbook.yml" --ident job --json \
    > "$job/runner.log" 2>&1 || result=$?
# Where each platform's binaries came from: CI's build, a release, or a
# source build with the reason no artifact was used.
for note in "$job"/build-*.txt; do
    [[ -f $note ]] && sed 's/^/Binaries: /' "$note" >&2
done
for target in "$@"; do
    report="$job/$target.json"
    if [[ -f "$job/$target.desktop.txt" ]]; then sed "s/^/Desktop app on $target: /" "$job/$target.desktop.txt" >&2; fi
    # Processes that were still running the replaced binary. The private file
    # keeps their command lines; only the program and its command are shown.
    if [[ -s "$job/$target.processes.tsv" ]]; then
        jq -Rn --arg target "$target" '{target:$target, stale_processes: [inputs | split("\t") | select(length >= 5) | {
            state: .[0], pid: (.[1] | tonumber? // 0), supervisor: .[2], unit: .[3],
            command: (.[4] | split(" ") | [(.[0] | split("/") | last)] + [.[1:][] | select(test("^[a-z][a-z-]*$"))][:1] | join(" "))}]}' \
            "$job/$target.processes.tsv"
    fi
    if [[ -f "$report" ]]; then
        jq --arg target "$target" '{target:$target,state,error}
            + (if has("process") then {process,readiness,logs} else {} end)
            + (if has("findings") then {summary, findings: [.findings[] | select(.state != "ok")]} else {} end)' "$report"
        jq -e '.state == "succeeded" or .state == "skipped"' "$report" > /dev/null || result=1
    else
        jq -n --arg target "$target" '{target:$target,state:"unverified"}'
        result=1
    fi
done
printf 'Private reports: %s\n' "$job" >&2
exit "$result"
