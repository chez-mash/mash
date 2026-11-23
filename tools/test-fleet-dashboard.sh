#!/usr/bin/env bash
# A deploy restarts a declared, running peer dashboard and leaves a stopped one
# alone. Real Ansible tasks against disposable supervisor doubles; no host
# services are touched.
set -euo pipefail
repository=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
source "$repository/tools/controller-environment.sh"
scratch=$(mktemp -d "${TMPDIR:?}/fleet-dashboard.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/bin"
cat > "$scratch/bin/systemctl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "$*" >> "$FAKE_ACTIONS"
[[ ${1:-} == --user ]] && shift
case "$1" in
    is-active)
        if [[ -f "$FAKE_RUNNING" ]]; then echo active; else echo inactive; exit 3; fi
        ;;
    restart) touch "$FAKE_RUNNING" ;;
    *) exit 2 ;;
esac
SH
cat > "$scratch/bin/launchctl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "$*" >> "$FAKE_ACTIONS"
case "$1" in
    print)
        [[ -f "$FAKE_RUNNING" ]] || exit 113
        echo 'state = running'
        ;;
    kickstart) touch "$FAKE_RUNNING" ;;
    *) exit 2 ;;
esac
SH
chmod 755 "$scratch/bin/systemctl" "$scratch/bin/launchctl"
run() {
    local manager=$1
    cat > "$scratch/test.yml" <<YAML
- hosts: localhost
  connection: local
  gather_facts: false
  environment:
    PATH: "$scratch/bin:$PATH"
    FAKE_RUNNING: "$scratch/running"
    FAKE_ACTIONS: "$scratch/actions"
  vars:
    mash_dashboard_restart: {manager: $manager, unit: dev.mash.dash}
  tasks:
    - ansible.builtin.include_tasks: "$repository/deployment/playbooks/tasks/dashboard-restart.yml"
YAML
    rm -f "$scratch/actions"
    uv run --project "$repository/deployment" --locked --no-sync ansible-playbook \
        -i localhost, "$scratch/test.yml" > "$scratch/runner.log" 2>&1 || {
            tail -80 "$scratch/runner.log" >&2
            exit 1
        }
}
uid=$(id -u)
touch "$scratch/running"
run systemd_user
[[ $(cat "$scratch/actions") == $'--user is-active dev.mash.dash\n--user restart dev.mash.dash\n--user is-active dev.mash.dash' ]]
run systemd_system
[[ $(cat "$scratch/actions") == $'is-active dev.mash.dash\nrestart dev.mash.dash\nis-active dev.mash.dash' ]]
run launchd
[[ $(cat "$scratch/actions") == "print gui/$uid/dev.mash.dash"$'\n'"kickstart -k gui/$uid/dev.mash.dash"$'\n'"print gui/$uid/dev.mash.dash" ]]
# A stopped dashboard stays stopped, and that is not a failed deploy.
rm "$scratch/running"
run systemd_user
[[ $(cat "$scratch/actions") == '--user is-active dev.mash.dash' ]]
run launchd
[[ $(cat "$scratch/actions") == "print gui/$uid/dev.mash.dash" ]]
[[ ! -e "$scratch/running" ]]
echo 'Deploy restarts a running declared dashboard and leaves a stopped one alone (supervisor doubles).'
