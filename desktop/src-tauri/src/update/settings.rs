//! What the app remembers about updates, in a small file of its own beside
//! its other data. Mash's configuration (`config.toml`) belongs to the
//! engine and is never read or written here.

use super::channel::Channel;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub channel: Channel,
    /// A version the person went back from, which is not installed again.
    pub skipped_version: Option<String>,
    /// The person chose to keep a separate `mash` command; the app no longer
    /// offers to use its own everywhere.
    pub keep_separate: bool,
    /// An update that was installed and has not yet started once.
    pub pending: Option<Pending>,
}

/// An installed update the app has not yet seen running.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    /// The version that installed it, and the one it installed.
    pub from: String,
    pub to: String,
    /// The copy of the previous app, kept until the new one has started.
    pub previous: Option<PathBuf>,
}

impl Settings {
    /// The settings in `file`, or the defaults when it is missing or was
    /// written by something this version cannot read.
    pub fn load(file: &Path) -> Self {
        std::fs::read(file)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Write the settings so that a crash leaves the old file or the new one.
    pub fn save(&self, file: &Path) -> std::io::Result<()> {
        if let Some(folder) = file.parent() {
            std::fs::create_dir_all(folder)?;
        }
        let draft = file.with_extension("json.new");
        std::fs::write(&draft, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(draft, file)
    }
}

#[cfg(test)]
mod tests {
    use super::{Pending, Settings};
    use crate::update::channel::Channel;

    #[test]
    fn settings_survive_a_round_trip_and_a_damaged_file() {
        let folder = std::env::temp_dir().join(format!("mash-settings-{}", std::process::id()));
        let file = folder.join("updates.json");
        assert_eq!(Settings::load(&file), Settings::default());
        let settings = Settings {
            channel: Channel::Main,
            skipped_version: Some("0.2.5-main.3".into()),
            keep_separate: true,
            pending: Some(Pending {
                from: "0.2.4".into(),
                to: "0.2.5-main.4".into(),
                previous: Some(folder.join("previous/Mash.app")),
            }),
        };
        settings.save(&file).unwrap();
        assert_eq!(Settings::load(&file), settings);
        // A file from an older version: only what it names is taken.
        std::fs::write(&file, r#"{"channel":"main"}"#).unwrap();
        assert_eq!(Settings::load(&file).channel, Channel::Main);
        // Earlier versions could install while idle. What they saved about
        // that is passed over, and the rest of the file is still read.
        let earlier = r#"{"channel":"main","install_when_idle":true,"keep_separate":true,
            "pending":{"from":"0.2.4","to":"0.2.5","previous":null,"quiet":true}}"#;
        std::fs::write(&file, earlier).unwrap();
        let read = Settings::load(&file);
        assert_eq!(read.channel, Channel::Main);
        assert!(read.keep_separate);
        assert_eq!(read.pending.map(|pending| pending.to), Some("0.2.5".into()));
        std::fs::write(&file, "not json").unwrap();
        assert_eq!(Settings::load(&file), Settings::default());
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
