#!/usr/bin/env bash
# Live previews of a worktree's front ends, for looking at a change from
# another device on the tailnet seconds after it is made. `tools/worktree.sh`
# runs this file; use it through that command:
#
#   tools/worktree.sh preview NAME [--site] [--real] [--local] [--stop]
#   tools/worktree.sh previews
#
# A preview is the package's own dev server (Vite for the dashboard, Astro
# for the website), started from the worktree as a transient user unit
# `mash-preview-NAME[-site]` with a memory cap, so it outlives the shell that
# started it and stops with one command. It listens on this machine's
# Tailscale address only, never on every interface, and `--local` uses
# 127.0.0.1 instead. Its port is chosen from the worktree's name inside a
# reserved range and kept for as long as the preview is recorded.
#
# The dashboard shows the demo's made-up history unless `--real` is given,
# which forwards reads to this machine's running `mash dash`: anyone on the
# tailnet who opens the address then sees real chat history.
#
# The addresses go to standard output and everything else to standard error.
# State and logs live in $XDG_STATE_HOME/mash/previews (MASH_PREVIEW_STATE
# moves them), outside the worktree so that it stays clean for `done`.
set -euo pipefail

say() { printf '%s\n' "$*" >&2; }
die() { say "preview: $*"; exit 1; }

state=${MASH_PREVIEW_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/mash/previews}
memory_cap=${MASH_PREVIEW_MEMORY_MAX:-1500M}
# One hundred ports for each kind of preview.
dashboard_ports=5200
site_ports=5300
storybook_ports=5400

# STORYBOOK HOOK: Storybook does not exist in this repository yet. When it
# does, start it here the way the dashboard is started, in storybook_ports
# under the unit mash-preview-NAME-storybook, and remove this refusal.
storybook_missing() {
    die "Storybook is not set up yet; ports $storybook_ports-$((storybook_ports + 99)) are reserved for its previews"
}

unit_for() { # NAME KIND
    if [[ $2 == dashboard ]]; then printf 'mash-preview-%s\n' "$1"; else printf 'mash-preview-%s-%s\n' "$1" "$2"; fi
}
record_of() { printf '%s/%s.state\n' "${state:?}" "${1:?}"; }
recorded() { # UNIT KEY
    [[ -f $(record_of "$1") ]] || return 0
    sed -n "s/^$2=//p" "$(record_of "$1")" | head -n 1
}
forget() { rm -f "$(record_of "$1")"; }
# Whether the unit's record is for another worktree or kind: a worktree named
# x-site and the website of a worktree named x would share a unit name.
taken_by_another() { # UNIT NAME KIND
    [[ -f $(record_of "$1") && ($(recorded "$1" name) != "$2" || $(recorded "$1" kind) != "$3") ]]
}
active() { systemctl --user is-active --quiet "$1.service" 2> /dev/null; }
answers() { curl --silent --output /dev/null --max-time 2 "$1" 2> /dev/null; }
# The option that selects a kind, for the commands shown in messages.
kind_option() { [[ $1 == dashboard ]] || printf ' --%s' "$1"; }

# This machine's Tailscale IPv4 address. Tailscale hands out addresses in
# 100.64.0.0/10 only, so anything else is not an address to share history on.
tailnet_address() {
    local address
    address=$(tailscale ip -4 2> /dev/null | head -n 1) || true
    [[ $address =~ ^100\.(6[4-9]|[7-9][0-9]|1[01][0-9]|12[0-7])\.[0-9]+\.[0-9]+$ ]] ||
        die 'Tailscale is not up (tailscale ip -4 gave no tailnet address); start it, or pass --local to preview on 127.0.0.1 only'
    printf '%s\n' "$address"
}
# The MagicDNS name, such as xenia.example.ts.net, or nothing without one.
tailnet_name() {
    tailscale status --json 2> /dev/null |
        python3 -c 'import json, sys; print((json.load(sys.stdin).get("Self") or {}).get("DNSName", "").rstrip("."))' 2> /dev/null || true
}

# Whether nothing listens on the port and no other recorded preview holds it.
port_free() { # PORT UNIT
    local file
    [[ -z $(ss -Hltn "sport = :$1" 2> /dev/null) ]] || return 1
    for file in "$state"/*.state; do
        [[ -f $file && $file != "$(record_of "$2")" ]] || continue
        [[ $(sed -n 's/^port=//p' "$file") != "$1" ]] || return 1
    done
}
# The port this preview had before if it is still free; otherwise the slot
# its name hashes to, or the next free one after it, wrapping in the range.
choose_port() { # NAME UNIT FIRST
    local previous slot step candidate
    previous=$(recorded "$2" port)
    if [[ -n $previous ]] && port_free "$previous" "$2"; then
        printf '%s\n' "$previous"
        return
    fi
    slot=$(($(printf '%s' "$1" | cksum | cut -d ' ' -f 1) % 100))
    for step in $(seq 0 99); do
        candidate=$(($3 + (slot + step) % 100))
        if port_free "$candidate" "$2"; then
            printf '%s\n' "$candidate"
            return
        fi
    done
    die "no free port in $3-$(($3 + 99)); stop a preview first (tools/worktree.sh previews)"
}

print_addresses() { # UNIT
    local host port path name
    host=$(recorded "$1" host) port=$(recorded "$1" port) path=$(recorded "$1" path) name=$(recorded "$1" dns)
    printf 'http://%s:%s%s\n' "$host" "$port" "$path"
    [[ -z $name ]] || printf 'http://%s:%s%s\n' "$name" "$port" "$path"
}

# Where `--real` reads from, and the file holding that dashboard's access
# token when it asks for one. `mash dash remote` writes both: the unit
# mash-dash.service, and MASH_DASH_TOKEN in ~/.mash/dashboard.env. The token
# is never read here; systemd hands the file to the dev server, whose proxy
# presents it (see dashboard/vite.config.ts).
real_dashboard() {
    local listen
    real_target=${MASH_DASHBOARD_API_URL:-}
    real_token_file=${MASH_PREVIEW_DASH_ENV:-$HOME/.mash/dashboard.env}
    if [[ -z $real_target ]]; then
        listen=$(systemctl --user show mash-dash.service --property ExecStart --value 2> /dev/null |
            grep -oE -- '--listen [^ ;"]+' | head -n 1 | cut -d ' ' -f 2) || true
        real_target=http://${listen:-127.0.0.1:8790}
    fi
    [[ -r $real_token_file ]] || real_token_file=''
}

start() { # NAME TREE KIND DATA BIND
    local name=$1 tree=$2 kind=$3 data=$4 bind=$5
    local unit package first path='/' host dns='' port log script url waited=0
    unit=$(unit_for "$name" "$kind")
    [[ -d $tree ]] || die "no worktree at $tree; see tools/worktree.sh list"
    mkdir -p "$state"
    ! taken_by_another "$unit" "$name" "$kind" ||
        die "the unit name $unit belongs to the $(recorded "$unit" kind) preview of $(recorded "$unit" name); stop that one first"
    if active "$unit"; then
        say "The $kind preview of $name is already running ($(recorded "$unit" data)). Stop it with: tools/worktree.sh preview $name$(kind_option "$kind") --stop"
        print_addresses "$unit"
        return
    fi
    case "$kind" in
        dashboard) package=dashboard first=$dashboard_ports ;;
        site) package=site first=$site_ports ;;
        *) die "unknown kind $kind" ;;
    esac
    command -v systemd-run > /dev/null || die 'systemd-run is needed to keep a preview running; this is a Linux tool'
    if [[ $bind == local ]]; then
        host=127.0.0.1
    else
        host=$(tailnet_address)
        dns=$(tailnet_name)
    fi
    if [[ ! -d $tree/$package/node_modules ]]; then
        say "Installing $package dependencies"
        (cd "$tree/$package" && pnpm install --frozen-lockfile --prefer-offline >&2) || die "pnpm install failed in $tree/$package"
    fi
    port=$(choose_port "$name" "$unit" "$first")
    log=$state/$unit.log
    : > "$log"
    # The dev servers refuse a Host header they were not told about, which
    # protects against DNS rebinding. The address itself always passes; the
    # MagicDNS name is added for this run only, from outside the committed
    # configuration: Vite reads the variable and Astro takes a flag.
    local properties=(--property "MemoryMax=$memory_cap" --property MemorySwapMax=0
        --property "WorkingDirectory=$tree/$package"
        --property "StandardOutput=append:$log" --property "StandardError=append:$log"
        --setenv "PATH=$PATH" --setenv "HOME=$HOME")
    local arguments=(--host "$host" --port "$port")
    if [[ $kind == dashboard ]]; then
        arguments+=(--strictPort)
        [[ -z $dns ]] || properties+=(--setenv "__VITE_ADDITIONAL_SERVER_ALLOWED_HOSTS=$dns")
        if [[ $data == real ]]; then
            script=dev
            real_dashboard
            properties+=(--setenv "MASH_DASHBOARD_API_URL=$real_target" --setenv MASH_PREVIEW_READ_ONLY=1)
            [[ -z $real_token_file ]] || properties+=(--property "EnvironmentFile=$real_token_file")
        else
            script=dev:demo
            path=/demo/
        fi
    else
        script=dev
        [[ -z $dns ]] || arguments+=(--allowed-hosts "$dns")
    fi
    printf '%s\n' "name=$name" "kind=$kind" "tree=$tree" "host=$host" "dns=$dns" "port=$port" "path=$path" \
        "data=$data" "started=$(date +%s)" > "$(record_of "$unit")"
    systemd-run --user --quiet --collect --unit "$unit" --description "Mash $kind preview of $name" \
        "${properties[@]}" "$(command -v pnpm)" run "$script" "${arguments[@]}" >&2 ||
        { forget "$unit"; die "systemd-run could not start $unit"; }
    url=http://$host:$port$path
    until answers "$url"; do
        active "$unit" || { tail -n 20 "$log" >&2; forget "$unit"; die "$unit stopped while starting; its log is $log"; }
        waited=$((waited + 1))
        [[ $waited -lt ${MASH_PREVIEW_WAIT:-120} ]] || { say "Started $unit, but it does not answer yet; watch $log"; break; }
        sleep 0.5
    done
    say "Started the $kind preview of $name as $unit ($data, at most $memory_cap of memory, log $log)."
    if [[ $data == real && $bind != local ]]; then
        say "REAL HISTORY: this preview reads $real_target. Anyone on the tailnet who opens the address can read"
        say 'this machine'"'"'s real chat history, without signing in, until the preview is stopped. It cannot change anything.'
    elif [[ $data == real ]]; then
        say "This preview reads real chat history from $real_target, without signing in; it cannot change anything."
    fi
    [[ $bind != local ]] || say 'Listening on 127.0.0.1 only; other devices cannot open it.'
    say "Edits in $tree/$package show up without a restart. Stop it with: tools/worktree.sh preview $name$(kind_option "$kind") --stop"
    print_addresses "$unit"
}

stop() { # NAME KIND
    local unit
    unit=$(unit_for "$1" "$2")
    ! taken_by_another "$unit" "$1" "$2" || die "no $2 preview of $1 is recorded"
    if active "$unit"; then
        systemctl --user stop "$unit.service" >&2 || die "could not stop $unit"
        say "Stopped $unit."
    else
        say "No $2 preview of $1 is running."
    fi
    systemctl --user reset-failed "$unit.service" 2> /dev/null || true
    forget "$unit"
}

# Every preview recorded for a worktree; `done` runs this before removing it.
stop_all() { # NAME
    local file
    for file in "$state"/*.state; do
        [[ -f $file && $(sed -n 's/^name=//p' "$file") == "$1" ]] || continue
        stop "$1" "$(sed -n 's/^kind=//p' "$file")"
    done
}

age() { # SECONDS
    if [[ $1 -lt 60 ]]; then
        printf '%ss\n' "$1"
    elif [[ $1 -lt 3600 ]]; then
        printf '%sm\n' $(($1 / 60))
    elif [[ $1 -lt 86400 ]]; then
        printf '%sh\n' $(($1 / 3600))
    else
        printf '%sd\n' $(($1 / 86400))
    fi
}

list() {
    local file unit status now found=no
    now=$(date +%s)
    for file in "$state"/*.state; do
        [[ -f $file ]] || continue
        [[ $found == yes ]] || printf '%-20s %-10s %-5s %-5s %-10s %s\n' NAME KIND DATA AGE STATE URL
        found=yes
        unit=$(basename "$file" .state)
        if ! active "$unit"; then
            status=stopped
        elif answers "$(print_addresses "$unit" | head -n 1)"; then
            status=answers
        else
            status=silent
        fi
        printf '%-20s %-10s %-5s %-5s %-10s %s\n' "$(recorded "$unit" name)" "$(recorded "$unit" kind)" \
            "$(recorded "$unit" data)" "$(age $((now - $(recorded "$unit" started))))" "$status" \
            "$(print_addresses "$unit" | tail -n 1)"
    done
    [[ $found == yes ]] || say 'No previews. Start one with: tools/worktree.sh preview NAME'
}

command_preview() { # NAME TREE [--site] [--storybook] [--real] [--local] [--stop]
    local name=$1 tree=$2 kind=dashboard data=demo bind=tailnet stopping=no
    shift 2
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --site) kind=site ;;
            --storybook) storybook_missing ;;
            --real) data=real ;;
            --local) bind=local ;;
            --stop) stopping=yes ;;
            *) die "unknown preview option $1 (--site, --real, --local, --stop)" ;;
        esac
        shift
    done
    if [[ $stopping == yes ]]; then
        stop "$name" "$kind"
        return
    fi
    # The website has no private data; it shows what the worktree holds.
    if [[ $kind == site ]]; then
        [[ $data == demo ]] || die '--real applies to the dashboard only'
        data=docs
    fi
    start "$name" "$tree" "$kind" "$data" "$bind"
}

operation=${1:-}
[[ $# -eq 0 ]] || shift
case "$operation" in
    preview)
        [[ $# -ge 2 ]] || die 'usage: preview NAME TREE [options]'
        command_preview "$@"
        ;;
    stop-all)
        [[ $# -eq 1 ]] || die 'usage: stop-all NAME'
        stop_all "$1"
        ;;
    list) list ;;
    *) die 'run this through tools/worktree.sh preview or previews' ;;
esac
