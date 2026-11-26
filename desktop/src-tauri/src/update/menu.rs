//! The update items of the app's two menus (the menu bar icon's and, on
//! macOS, the app menu): a line that says where things stand and, when a
//! newer version is on offer, does what the banner's Update and Restart
//! does; Check for Updates…; and the channel. Each menu has its own items,
//! kept here so one change of state reaches both.

use super::banner::Standing;
use super::channel::Channel;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Wry};

pub const STATUS: &str = "update-status";
pub const CHECK: &str = "update-check";
pub const STABLE: &str = "update-stable";
pub const MAIN: &str = "update-main";
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
}

/// The status line for where an update stands, and whether choosing it does
/// something. It says what the banner says: a version that is downloaded,
/// or still arriving, is offered with the banner's own action; once that is
/// chosen (`wanted`) the line follows the download and the install.
pub fn line(standing: Standing<'_>, wanted: bool, current: &str) -> (String, bool) {
    match standing {
        Standing::Installing(version) => (format!("Installing Mash {version}…"), false),
        Standing::Downloading(version) if wanted => (format!("Downloading Mash {version}…"), false),
        Standing::Downloading(version) | Standing::Ready(version) => (
            format!("Mash {version} Is Available — Update and Restart"),
            true,
        ),
        Standing::Off | Standing::Nothing => (format!("Mash {current}"), false),
    }
}

#[derive(Default)]
pub struct Items {
    status: Vec<MenuItem<Wry>>,
    channel: Vec<Submenu<Wry>>,
    stable: Vec<CheckMenuItem<Wry>>,
    main: Vec<CheckMenuItem<Wry>>,
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
            let channel = Submenu::with_id_and_items(
                app,
                "update-channel",
                "Updates",
                true,
                &[&stable, &main],
            )?;
            add(&channel)?;
            self.channel.push(channel);
            self.stable.push(stable);
            self.main.push(main);
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

#[cfg(test)]
mod tests {
    use super::{Standing, line};

    #[test]
    fn the_line_offers_what_the_banner_offers() {
        let offered = (
            "Mash 0.2.5 Is Available — Update and Restart".to_owned(),
            true,
        );
        assert_eq!(line(Standing::Ready("0.2.5"), false, "0.2.4"), offered);
        assert_eq!(
            line(Standing::Downloading("0.2.5"), false, "0.2.4"),
            offered
        );
        // A download that finished after the choice installs by itself.
        assert_eq!(line(Standing::Ready("0.2.5"), true, "0.2.4"), offered);
    }

    #[test]
    fn the_line_only_reports_while_the_app_is_at_work_or_has_nothing() {
        assert_eq!(
            line(Standing::Downloading("0.2.5"), true, "0.2.4"),
            ("Downloading Mash 0.2.5…".to_owned(), false)
        );
        assert_eq!(
            line(Standing::Installing("0.2.5"), false, "0.2.4"),
            ("Installing Mash 0.2.5…".to_owned(), false)
        );
        for standing in [Standing::Nothing, Standing::Off] {
            assert_eq!(
                line(standing, false, "0.2.4"),
                ("Mash 0.2.4".to_owned(), false)
            );
        }
    }
}
