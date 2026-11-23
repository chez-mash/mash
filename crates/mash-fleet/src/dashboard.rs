//! Which machines' dashboards a deploy restarts on the release it installed.
//!
//! Installing Mash replaces the binary but leaves a running dashboard on the
//! build it started from, so that dashboard cannot answer for the new release
//! (for example, reinstall the plugin the release ships) until it restarts.
//! A deploy restarts it only when the inventory already declares the
//! dashboard as a controllable service of the machine: the same opt-in the
//! services page uses for every other supervisor target. It never restarts
//! the controller's own dashboard, which is the one running the job, and it
//! does not restart a Windows dashboard, since the playbooks cannot run
//! commands on a Windows machine. Everywhere else the person restarts it.
use crate::inventory::{Inventory, Machine, Manager, Platform, ServiceKind};
use serde::Serialize;

/// The supervisor call a deploy makes to restart one dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Restart<'a> {
    /// `systemd_user`, `systemd_system` or `launchd`.
    pub manager: Manager,
    /// The unit name, or the launchd label.
    pub unit: &'a str,
}

/// The dashboard a deploy to `machine` restarts, if any.
#[must_use]
pub fn restarted_by_deploy<'a>(inventory: &Inventory, machine: &'a Machine) -> Option<Restart<'a>> {
    if machine.id == inventory.controller.node || machine.platform == Platform::Windows {
        return None;
    }
    let mut dashboards = machine
        .services
        .iter()
        .filter(|service| service.kind == ServiceKind::Dashboard);
    let service = dashboards.next()?;
    // Two dashboards on one machine leave no single one to restart.
    if dashboards.next().is_some() || !service.allow_control {
        return None;
    }
    matches!(
        service.manager,
        Manager::SystemdUser | Manager::SystemdSystem | Manager::Launchd
    )
    .then_some(Restart {
        manager: service.manager,
        unit: &service.unit,
    })
}

/// The machines whose dashboard a deploy restarts, in inventory order.
#[must_use]
pub fn restarted_machines(inventory: &Inventory) -> Vec<String> {
    inventory
        .machines
        .iter()
        .filter(|machine| machine.history && restarted_by_deploy(inventory, machine).is_some())
        .map(|machine| machine.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Service;

    fn dashboard(manager: Manager, allow_control: bool) -> Service {
        Service {
            id: "dashboard".into(),
            name: "Dashboard".into(),
            kind: ServiceKind::Dashboard,
            manager,
            unit: "mash-dash.service".into(),
            plist: None,
            log: None,
            allow_control,
            deployment: None,
        }
    }

    fn fleet() -> Inventory {
        let mut inventory = crate::inventory::tests::fixture();
        let mut peer = inventory.machines[0].clone();
        peer.id = "macbook".into();
        peer.services = vec![dashboard(Manager::SystemdUser, true)];
        inventory.machines.push(peer);
        inventory.validate().unwrap();
        inventory
    }

    #[test]
    fn deploy_restarts_a_peer_dashboard_declared_controllable() {
        let inventory = fleet();
        assert_eq!(
            restarted_by_deploy(&inventory, &inventory.machines[1]),
            Some(Restart {
                manager: Manager::SystemdUser,
                unit: "mash-dash.service"
            })
        );
        assert_eq!(restarted_machines(&inventory), ["macbook"]);
        // The playbook reads it from the host's variables.
        let document: serde_json::Value =
            serde_json::from_slice(&crate::ansible::encode(&inventory).unwrap()).unwrap();
        let hosts = &document["all"]["children"]["mash_history"]["hosts"];
        assert_eq!(
            hosts["macbook"]["mash_dashboard_restart"],
            serde_json::json!({"manager": "systemd_user", "unit": "mash-dash.service"})
        );
        assert!(hosts["xenia"].get("mash_dashboard_restart").is_none());
    }

    #[test]
    fn deploy_leaves_dashboards_it_was_not_given_alone() {
        // The controller's own dashboard runs the job.
        let mut inventory = fleet();
        inventory.machines[0].services = vec![dashboard(Manager::SystemdUser, true)];
        assert_eq!(
            restarted_by_deploy(&inventory, &inventory.machines[0]),
            None
        );
        // Listed but not controllable, or not listed at all.
        let mut inventory = fleet();
        inventory.machines[1].services = vec![dashboard(Manager::SystemdUser, false)];
        assert_eq!(restarted_machines(&inventory), Vec::<String>::new());
        inventory.machines[1].services.clear();
        assert_eq!(restarted_machines(&inventory), Vec::<String>::new());
        // A Windows machine, or a manager with no restart through the playbooks.
        let mut inventory = fleet();
        inventory.machines[1].platform = Platform::Windows;
        assert_eq!(restarted_machines(&inventory), Vec::<String>::new());
        let mut inventory = fleet();
        inventory.machines[1].services = vec![dashboard(Manager::WindowsService, true)];
        assert_eq!(restarted_machines(&inventory), Vec::<String>::new());
        // Two dashboards: which one serves the machine is not known.
        let mut inventory = fleet();
        let mut second = dashboard(Manager::SystemdUser, true);
        second.id = "dashboard-2".into();
        inventory.machines[1].services.push(second);
        assert_eq!(restarted_machines(&inventory), Vec::<String>::new());
    }
}
