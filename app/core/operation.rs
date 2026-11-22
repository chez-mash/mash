//! Facts about each operation that more than one caller needs. Both matches
//! are exhaustive, so a new operation cannot compile until it says how it is
//! logged and whether it changes anything.
use super::models::Operation;

impl Operation {
    /// The name its request is logged under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::RecordMutation(_) | Self::ExtractRecords(_) => "record_mutation",
            Self::Records(_)
            | Self::Record(_)
            | Self::RecallRecords(_)
            | Self::RecordCapture(_)
            | Self::RecordMatches(_)
            | Self::ExtractionCheckpoint(_) => "records",
            Self::Init(_) => "init",
            Self::Index(_) => "index",
            Self::ListChats { .. } | Self::DashboardCatalog { .. } => "list_chats",
            Self::Search { .. } => "search",
            Self::LiveSearch { .. } => "live_search",
            Self::View { .. } | Self::ChatMessages { .. } | Self::ChatContext { .. } => "view",
            Self::Status { .. } | Self::Installation { .. } => "status",
            Self::Doctor => "doctor",
            Self::Forget { .. } => "forget",
            Self::QuerySilver { .. } | Self::QueryStructured { .. } | Self::QueryShared { .. } => {
                "query"
            }
            Self::ModelsInstall => "models_install",
            Self::QdrantInstall => "qdrant_install",
            Self::QdrantUninstall { .. } => "qdrant_uninstall",
            Self::Uninstall(_) => "uninstall",
            Self::ExportSilver { .. } => "export_silver",
            Self::ImportSilver { .. } => "import_silver",
            Self::ListAttachments { .. } => "list_attachments",
            Self::GetAttachment { .. } => "get_attachment",
        }
    }

    /// How long it may run before it fails as timed out. Only searches have
    /// one: every search, including any wait for model capacity, ends by it
    /// rather than hanging, though blocking work already started may finish
    /// unobserved. A search run on each keystroke is worth nothing once the
    /// reader has typed on, so it gives up far sooner.
    pub fn deadline(&self) -> Option<std::time::Duration> {
        match self {
            Self::Search { .. } => Some(std::time::Duration::from_secs(15)),
            Self::LiveSearch { .. } => Some(std::time::Duration::from_secs(5)),
            _ => None,
        }
    }

    /// Whether it changes stored history, records, models, services or files,
    /// so a long-lived server runs it alone and only while it is current.
    /// Reads may run alongside one another.
    pub fn writes(&self) -> bool {
        match self {
            Self::Index(_)
            | Self::RecordMutation(_)
            | Self::ExtractRecords(_)
            | Self::Forget { .. }
            | Self::ModelsInstall
            | Self::QdrantInstall
            | Self::QdrantUninstall { .. }
            | Self::ExportSilver { .. }
            | Self::ImportSilver { .. }
            | Self::Init(_)
            | Self::Uninstall(_) => true,
            Self::Records(_)
            | Self::Record(_)
            | Self::RecallRecords(_)
            | Self::RecordCapture(_)
            | Self::RecordMatches(_)
            | Self::ExtractionCheckpoint(_)
            | Self::ListChats { .. }
            | Self::DashboardCatalog { .. }
            | Self::ChatContext { .. }
            | Self::Search { .. }
            | Self::LiveSearch { .. }
            | Self::View { .. }
            | Self::ChatMessages { .. }
            | Self::Status { .. }
            | Self::Doctor
            | Self::Installation { .. }
            | Self::QuerySilver { .. }
            | Self::QueryStructured { .. }
            | Self::QueryShared { .. }
            | Self::ListAttachments { .. }
            | Self::GetAttachment { .. } => false,
        }
    }
}
