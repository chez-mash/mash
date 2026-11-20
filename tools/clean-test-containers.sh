#!/usr/bin/env bash
# Remove the Docker containers Mash's tests leave behind: every container
# labelled mash.test (the per-test Qdrant containers and the restart
# fixture's), with their anonymous volumes, except a per-test container whose
# test process is still running. A real Qdrant carries no such label, so it is never
# touched. Pass an owner to remove only one run's containers, as CI does.
set -euo pipefail
filter=(--filter label=mash.test)
[[ $# -le 1 ]] || { echo 'usage: clean-test-containers.sh [OWNER]' >&2; exit 2; }
[[ -z ${1:-} ]] || filter+=(--filter "label=mash.test.owner=$1")
# MASH_TEST_DOCKER_SG=1 reaches Docker through the docker group, as on Xenia.
docker_cli() {
    if [[ ${MASH_TEST_DOCKER_SG:-} == 1 ]]; then
        sg docker -c "$(printf '%q ' docker "$@")"
    else
        docker "$@"
    fi
}
# A per-test Qdrant is owned by local-PID; while that test process runs, its
# container is in use, not leaked.
ids=()
while read -r id owner; do
    if [[ $owner == local-* ]] && kill -0 "${owner#local-}" 2>/dev/null; then
        continue
    fi
    ids+=("$id")
done < <(docker_cli ps -a "${filter[@]}" --format '{{.ID}} {{.Label "mash.test.owner"}}')
if (( ${#ids[@]} == 0 )); then
    echo 'No Mash test containers to remove.'
    exit 0
fi
docker_cli ps -a --format '{{.Names}}\t{{.Image}}\t{{.Status}}' "${ids[@]/#/--filter=id=}"
docker_cli rm -f -v "${ids[@]}" >/dev/null
echo "Removed ${#ids[@]} Mash test container(s)."
