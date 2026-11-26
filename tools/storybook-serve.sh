#!/usr/bin/env bash
# Serves the Storybook of `main` on the tailnet, as a reference that is always
# there: the dashboard's tokens, kit controls and building blocks in their
# states, with made-up data.
#
#   tools/storybook-serve.sh            build main's Storybook and serve it (or refresh it)
#   tools/storybook-serve.sh --status   what is served, and whether it answers
#   tools/storybook-serve.sh --stop     stop serving and remove the unit
#
# It builds in the deploy tree (`tools/worktree.sh deploy-tree`, a clean
# checkout of origin/main), copies the static files out of it, and serves the
# copy with the user unit `mash-storybook.service` on this machine's Tailscale
# address at port 5400: never on every interface, and nothing public. The unit
# is enabled, so it comes back after a restart of the machine.
#
# Nothing refreshes it by itself. Run this again after landing a change to
# the dashboard to serve the new main; the old files are served until the new
# build has finished, and a failed build leaves them in place.
#
# A worktree's own Storybook, live while you edit, is a preview instead:
# tools/worktree.sh preview NAME --storybook.
set -euo pipefail

usage() {
    sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
}
say() { printf '%s\n' "$*" >&2; }
die() { say "storybook: $*"; exit 1; }

tools=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
unit=mash-storybook
port=5400
state=${MASH_STORYBOOK_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/mash/storybook}
units=${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user

# This machine's Tailscale IPv4 address; anything outside 100.64.0.0/10 is not one.
tailnet_address() {
    local address
    address=$(tailscale ip -4 2> /dev/null | head -n 1) || true
    [[ $address =~ ^100\.(6[4-9]|[7-9][0-9]|1[01][0-9]|12[0-7])\.[0-9]+\.[0-9]+$ ]] ||
        die 'Tailscale is not up (tailscale ip -4 gave no tailnet address); the Storybook is served on the tailnet only'
    printf '%s\n' "$address"
}
tailnet_name() {
    tailscale status --json 2> /dev/null |
        python3 -c 'import json, sys; print((json.load(sys.stdin).get("Self") or {}).get("DNSName", "").rstrip("."))' 2> /dev/null || true
}
active() { systemctl --user is-active --quiet "$unit.service" 2> /dev/null; }
answers() { curl --silent --output /dev/null --max-time 2 "$1" 2> /dev/null; }
print_addresses() { # HOST
    local name
    name=$(tailnet_name)
    printf 'http://%s:%s/\n' "$1" "$port"
    [[ -z $name ]] || printf 'http://%s:%s/\n' "$name" "$port"
}

status() {
    local host
    if ! active; then
        say "$unit.service is not running. Start it with: tools/storybook-serve.sh"
        return 1
    fi
    host=$(tailnet_address)
    say "Serving main at $(cat "$state/revision" 2> /dev/null || echo 'an unknown revision') from $state/site."
    answers "http://$host:$port/index.json" || say "It does not answer; see: journalctl --user -u $unit"
    print_addresses "$host"
}

stop() {
    systemctl --user disable --now "$unit.service" > /dev/null 2>&1 || true
    rm -f "${units:?}/${unit:?}.service"
    systemctl --user daemon-reload 2> /dev/null || true
    say "Stopped $unit.service and removed its unit. The built files stay in $state/site."
}

serve() {
    local host tree revision waited=0 runner=()
    command -v systemctl > /dev/null || die 'systemd is needed to keep the Storybook served; this is a Linux tool'
    host=$(tailnet_address)
    # Another listener on the port, such as a Storybook preview that started
    # first, would keep the unit from binding.
    if ! active && [[ -n $(ss -Hltn "sport = :$port" 2> /dev/null) ]]; then
        die "port $port is taken; stop what holds it (tools/worktree.sh previews) and run this again"
    fi
    tree=$(bash "$tools/worktree.sh" deploy-tree) || die 'could not prepare the deploy tree'
    revision=$(git -C "$tree" rev-parse --short HEAD)
    [[ -f $tree/dashboard/.storybook/main.ts ]] || die "main at $revision has no Storybook yet"
    say "Building the Storybook of main at $revision"
    (cd "$tree/dashboard" && pnpm install --frozen-lockfile --prefer-offline >&2) || die "pnpm install failed in $tree/dashboard"
    # On Xenia heavy work runs under the shared test budget.
    [[ ! -x $HOME/.local/bin/agent-test ]] || runner=("$HOME/.local/bin/agent-test")
    (cd "$tree/dashboard" && "${runner[@]}" pnpm storybook:build >&2) || die 'the Storybook build failed; what was served before is still served'
    [[ -f $tree/dashboard/storybook-static/index.json ]] || die 'the build left no storybook-static/index.json'
    # The server reads a copy, so the deploy tree can move on, and the copy is
    # swapped in whole: a reader never gets half of one build and half of another.
    mkdir -p "$state"
    rm -rf "${state:?}/site.new" "${state:?}/site.old"
    cp -R "$tree/dashboard/storybook-static" "$state/site.new"
    [[ ! -d $state/site ]] || mv "$state/site" "$state/site.old"
    mv "$state/site.new" "$state/site"
    rm -rf "${state:?}/site.old"
    printf '%s\n' "$revision" > "$state/revision"

    mkdir -p "$units"
    cat > "$units/$unit.service" << UNIT
[Unit]
Description=Mash Storybook of main (static files, tailnet only)

[Service]
# Bound to the Tailscale address alone. Until Tailscale is up after a restart
# of the machine the address cannot be bound, so the unit tries again.
ExecStart=$(command -v python3) -m http.server $port --bind $host --directory $state/site
Restart=always
RestartSec=15
MemoryMax=200M
MemorySwapMax=0

[Install]
WantedBy=default.target
UNIT
    systemctl --user daemon-reload || die 'systemctl --user daemon-reload failed'
    systemctl --user enable --quiet "$unit.service" || die "could not enable $unit.service"
    systemctl --user restart "$unit.service" || die "could not start $unit.service"
    until answers "http://$host:$port/index.json"; do
        waited=$((waited + 1))
        [[ $waited -lt ${MASH_STORYBOOK_WAIT:-40} ]] || { say "Started $unit.service, but it does not answer yet; see: journalctl --user -u $unit"; break; }
        sleep 0.5
    done
    say "Serving the Storybook of main at $revision as $unit.service. Refresh it by running this again; stop it with --stop."
    print_addresses "$host"
}

case "${1:-}" in
    '') [[ $# -eq 0 ]] || usage; serve ;;
    --status) [[ $# -eq 1 ]] || usage; status ;;
    --stop) [[ $# -eq 1 ]] || usage; stop ;;
    *) usage ;;
esac
