#!/usr/bin/env bash
# Runs on the machine a deployment just installed to (Linux or macOS, bash 3.2
# or later). Lists this user's processes that still run a replaced Mash binary
# and restarts the ones a supervisor owns.
#
#   restart-stale-processes.sh [--dry-run] --binary INSTALLED_PATH...
#       [--restart MATCH START_COMMAND]...
#
# A process is stale when its executable is one of the installed paths but not
# the file now at that path: on Linux the two differ by inode, on macOS the
# process started before the file was put in place. Programs elsewhere, such
# as a development build or the desktop app's own engine, are never touched.
#
# One line per stale process, tab separated: STATE PID SUPERVISOR UNIT COMMAND.
#   restarted        a systemd user unit or launchd job whose main process it
#                    was, restarted through that supervisor
#   restart-failed   the same, when the supervisor did not bring it back
#   hand-started     started from a shell or by another program (an agent's
#                    `mash mcp`, a `mash dash` in a terminal): left running
#                    on the old binary for its owner to restart
#   unit-child       inside a unit whose main process is something else, such
#                    as an agent: restarting that unit is not this script's call
#   system-unit      a system service, which needs its administrator
#   restarted-by-inventory / restart-failed
#                    a hand-started process named by --restart: stopped, then
#                    started again with the given command
set -uo pipefail
dry_run=no
binaries=()
matches=()
starts=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) dry_run=yes; shift ;;
        --binary) [[ $# -ge 2 && $2 == /* ]] || { echo '--binary needs an absolute path' >&2; exit 2; }
            binaries+=("$2"); shift 2 ;;
        --restart) [[ $# -ge 3 && -n $2 && -n $3 ]] || { echo '--restart needs MATCH and START_COMMAND' >&2; exit 2; }
            matches+=("$2"); starts+=("$3"); shift 3 ;;
        *) echo 'usage: restart-stale-processes.sh [--dry-run] --binary PATH... [--restart MATCH START]...' >&2; exit 2 ;;
    esac
done
[[ ${#binaries[@]} -gt 0 ]] || { echo 'name at least one --binary' >&2; exit 2; }
os=$(uname -s)
proc=${MASH_STALE_PROC_ROOT:-/proc}
uid=$(id -u)

installed() {
    local binary
    for binary in "${binaries[@]}"; do
        [[ $1 == "$binary" ]] && return 0
    done
    return 1
}

# Prints "PID" for every stale process of this user.
stale_linux() {
    local entry exe path pid
    for entry in "$proc"/[0-9]*; do
        exe=$(readlink "$entry/exe" 2>/dev/null) || continue
        path=${exe% (deleted)}
        installed "$path" || continue
        pid=${entry##*/}
        # A replaced file is unlinked, so the link says so; the inode check
        # also covers a file written over in place.
        if [[ $exe == *' (deleted)' || ! -e $path ]] ||
            [[ $(stat -Lc %d:%i "$entry/exe" 2>/dev/null) != "$(stat -c %d:%i "$path" 2>/dev/null)" ]]; then
            echo "$pid"
        fi
    done
}
stale_darwin() {
    local pid started path placed
    # comm is the executable's full path on macOS; lstart is five words.
    ps -x -o pid=,lstart=,comm= | while read -r pid w1 w2 w3 w4 w5 path; do
        installed "$path" || continue
        # Renaming the new file into place changes its inode's change time.
        placed=$(stat -f %c "$path" 2>/dev/null) || continue
        started=$(date -j -f '%a %b %d %T %Y' "$w1 $w2 $w3 $w4 $w5" +%s 2>/dev/null) || continue
        [[ $started -lt $placed ]] && echo "$pid"
    done
}
command_line() {
    ps -o args= -p "$1" 2>/dev/null | tr '\t' ' ' | cut -c1-160
}
alive() { kill -0 "$1" 2>/dev/null; }
restarted_units=' '
report() { printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "$5"; }

handle_hand_started() {
    local pid=$1 command=$2 index state waited
    for ((index = 0; index < ${#matches[@]}; index++)); do
        [[ $command == *"${matches[$index]}"* ]] || continue
        if [[ $dry_run == yes ]]; then report would-restart "$pid" inventory - "$command"; return; fi
        kill -TERM "$pid" 2>/dev/null
        for ((waited = 0; waited < 20; waited++)); do alive "$pid" || break; sleep 0.5; done
        state=restart-failed
        if ! alive "$pid"; then
            # Detached, so it outlives this script's session.
            if command -v setsid >/dev/null; then
                setsid sh -c "${starts[$index]}" >/dev/null 2>&1 < /dev/null &
            else
                nohup sh -c "${starts[$index]}" >/dev/null 2>&1 < /dev/null &
            fi
            state=restarted-by-inventory
        fi
        report "$state" "$pid" inventory - "$command"
        return
    done
    report hand-started "$pid" - - "$command"
}

handle_linux() {
    local pid=$1 command cgroup unit main state
    command=$(command_line "$pid")
    cgroup=$(sed -n 's/^0:://p' "$proc/$pid/cgroup" 2>/dev/null)
    unit=${cgroup##*/}
    if [[ $unit != *.service ]]; then handle_hand_started "$pid" "$command"; return; fi
    if [[ $cgroup != */user@*.service/* ]]; then report system-unit "$pid" systemd "$unit" "$command"; return; fi
    # Only a unit that runs Mash itself. A `mash mcp` inside an agent's own
    # service shares that unit's group, and the agent must not be restarted.
    main=$(systemctl --user show "$unit" --property MainPID --value 2>/dev/null)
    if [[ $main != "$pid" ]]; then report unit-child "$pid" systemd "$unit" "$command"; return; fi
    if [[ $dry_run == yes ]]; then report would-restart "$pid" systemd "$unit" "$command"; return; fi
    [[ $restarted_units == *" $unit "* ]] && return
    restarted_units+="$unit "
    state=restart-failed
    if systemctl --user restart "$unit" >/dev/null 2>&1 && systemctl --user is-active --quiet "$unit"; then state=restarted; fi
    report "$state" "$pid" systemd "$unit" "$command"
}

handle_darwin() {
    local pid=$1 command label state
    command=$(command_line "$pid")
    # launchctl lists each job's own process; apps opened from the Finder
    # appear as application.* and are not services.
    label=$(launchctl list 2>/dev/null | awk -v pid="$pid" '$1 == pid { print $3 }')
    if [[ -z $label || $label == application.* ]]; then handle_hand_started "$pid" "$command"; return; fi
    if [[ $dry_run == yes ]]; then report would-restart "$pid" launchd "$label" "$command"; return; fi
    state=restart-failed
    if launchctl kickstart -k "gui/$uid/$label" >/dev/null 2>&1; then state=restarted; fi
    report "$state" "$pid" launchd "$label" "$command"
}

case "$os" in
    Linux) for pid in $(stale_linux); do handle_linux "$pid"; done ;;
    Darwin) for pid in $(stale_darwin); do handle_darwin "$pid"; done ;;
    *) echo "unsupported platform $os" >&2; exit 2 ;;
esac
exit 0
