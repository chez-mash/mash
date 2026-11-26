//! The update items of the app's two menus (the menu bar icon's and, on
//! macOS, the app menu): a line that says where things stand and restarts
//! into a ready update, Check for Updates…, and the channel with its idle
//! option. Each menu has its own items, kept here so one change of state
//! reaches both.

use super::channel::Channel;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Wry};

pub const STATUS: &str = "update-status";
pub const CHECK: &str = "update-check";
pub const STABLE: &str = "update-stable";
pub const MAIN: &str = "update-main";
pub const IDLE: &str = "update-idle";
pub const BACK: &str = "update-back";

/// A menu the update items live in.
pub enum Holder {
    Menu(Menu<Wry>),
    Submenu(Submenu<Wry>),
}

impl Holder {
    fn insert(&self, item: &dyn IsMenuItem<Wry>, position: Option<usize>) -> tauri::Result<()> {
        match (self, position) {
            (Self::Menu(menu), Some(position)) => menu.insert(item, position),
            (Self::Menu(menu), None) => menu.append(item),
            (Self::Submenu(menu), Some(position)) => menu.insert(item, position),
            (Self::Submenu(menu), None) => menu.append(item),
        }
    }

    /// Where the item called `id` is, to put another after it.
    fn position(&self, id: &str) -> Option<usize> {
        let items = match self {
            Self::Menu(menu) => menu.items(),
            Self::Submenu(menu) => menu.items(),
        };
        items.ok()?.iter().position(|item| item.id().as_ref() == id)
    }
}

/// What the items show.
pub struct View {
    /// The status line, and whether choosing it does something.
    pub status: String,
    pub actionable: bool,
    pub channel: Channel,
    pub installs_when_idle: bool,
}

#[derive(Default)]
pub struct Items {
    status: Vec<MenuItem<Wry>>,
    channel: Vec<Submenu<Wry>>,
    stable: Vec<CheckMenuItem<Wry>>,
    main: Vec<CheckMenuItem<Wry>>,
    idle: Vec<CheckMenuItem<Wry>>,
    holders: Vec<Holder>,
    going_back: bool,
}

impl Items {
    /// Add the update items to a menu, at `position` or at its end. With
    /// updates off (`channels` false) there is only Check for Updates…, which
    /// then opens the downloads page.
    pub fn add_to(
        &mut self,
        app: &AppHandle,
        holder: Holder,
        position: Option<usize>,
        channels: bool,
    ) -> tauri::Result<()> {
        let mut at = position;
        let mut add = |item: &dyn IsMenuItem<Wry>| {
            holder.insert(item, at)?;
            at = at.map(|position| position + 1);
            tauri::Result::Ok(())
        };
        add(&PredefinedMenuItem::separator(app)?)?;
        if channels {
            let status = MenuItem::with_id(app, STATUS, "Mash", false, None::<&str>)?;
            add(&status)?;
            self.status.push(status);
        }
        add(&MenuItem::with_id(
            app,
            CHECK,
            "Check for Updates…",
            true,
            None::<&str>,
        )?)?;
        if channels {
            let stable = CheckMenuItem::with_id(
                app,
                STABLE,
                Channel::Stable.label(),
                true,
                false,
                None::<&str>,
            )?;
            let main = CheckMenuItem::with_id(
                app,
                MAIN,
                Channel::Main.label(),
                true,
                false,
                None::<&str>,
            )?;
            let idle = CheckMenuItem::with_id(
                app,
                IDLE,
                "Install Automatically When Idle",
                true,
                false,
                None::<&str>,
            )?;
            let channel = Submenu::with_id_and_items(
                app,
                "update-channel",
                "Updates",
                true,
                &[&stable, &main, &PredefinedMenuItem::separator(app)?, &idle],
            )?;
            add(&channel)?;
            self.channel.push(channel);
            self.stable.push(stable);
            self.main.push(main);
            self.idle.push(idle);
        }
        self.holders.push(holder);
        Ok(())
    }

    pub fn show(&self, view: &View) {
        for status in &self.status {
            let _ = status.set_text(&view.status);
            let _ = status.set_enabled(view.actionable);
        }
        for channel in &self.channel {
            let _ = channel.set_text(format!("Updates: {}", view.channel.label()));
        }
        // A click toggles a check item by itself, so every one is set again.
        for stable in &self.stable {
            let _ = stable.set_checked(view.channel == Channel::Stable);
        }
        for main in &self.main {
            let _ = main.set_checked(view.channel == Channel::Main);
        }
        for idle in &self.idle {
            let _ = idle.set_checked(view.installs_when_idle);
        }
    }

    /// Add Go Back to Previous Version after the channel, once: only an
    /// update whose engine did not start offers it.
    pub fn offer_going_back(&mut self, app: &AppHandle) {
        if std::mem::replace(&mut self.going_back, true) {
            return;
        }
        for holder in &self.holders {
            let after = holder
                .position("update-channel")
                .or_else(|| holder.position(CHECK));
            let Ok(item) =
                MenuItem::with_id(app, BACK, "Go Back to Previous Version", true, None::<&str>)
            else {
                continue;
            };
            let _ = holder.insert(&item, after.map(|position| position + 1));
        }
    }
}
