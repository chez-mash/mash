//! `mash qdrant …`: install, remove and tidy the local Qdrant search service.
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Clone, Subcommand)]
pub(super) enum QdrantCommand {
    /// Download, verify and start the local Qdrant search service.
    #[command(long_about = concat!(
        "Download, verify and start the local Qdrant search service.\n\n",
        "Downloads Qdrant 1.20.0 for this platform (about 30 MB) from ",
        "https://github.com/qdrant/qdrant/releases, checks it against the SHA-256 built into Mash, ",
        "and installs it in ~/.mash/qdrant. ",
        "It runs as your user, only on 127.0.0.1 at the ports in Mash's [qdrant] settings ",
        "(6333 and 6334 by default), and starts again at login: a systemd user service on Linux, ",
        "a LaunchAgent on macOS, a logon task on Windows. ",
        "Rerun it to repair or upgrade; Qdrant's data is kept. ",
        "If a compatible Qdrant already answers on those ports, nothing is installed. ",
        "Nothing else in Mash downloads or starts Qdrant.",
    ))]
    Install,
    /// Stop and remove the Qdrant service that `mash qdrant install` set up.
    Uninstall {
        /// Also delete Qdrant's data (Mash's search views; `mash index` rebuilds them from the index).
        #[arg(long)]
        delete_storage: bool,
    },
    /// List Mash's search collections in Qdrant that no index uses; with --yes, remove them.
    #[command(long_about = concat!(
        "List Mash's search collections in Qdrant that no index uses; with --yes, remove them.\n\n",
        "Every index and shared server marks its collections with an owner. Collections whose ",
        "owner database no longer exists are leftovers, and --yes removes them. Collections this ",
        "index or an --index uses, a shared server keeps, or whose owner sits in a folder that ",
        "cannot be read are kept. Collections from older Mash have no owner and are kept unless ",
        "you name them with --include-unknown. Without --yes, nothing is deleted.",
    ))]
    Prune {
        /// Another index whose collections are in use; repeat for each.
        #[arg(long = "index", value_name = "PATH")]
        indexes: Vec<PathBuf>,
        /// Also remove this group that has no owner: its hash or a collection name.
        #[arg(long = "include-unknown", value_name = "GROUP")]
        unknown: Vec<String>,
        /// Remove the groups the listing marks; without it, nothing is deleted.
        #[arg(long)]
        yes: bool,
    },
}
