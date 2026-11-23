//! The shared server's storage: its node configuration, the store that
//! commits silver and synchronizes views, and the hub that announces what a
//! publication changed. The HTTP service over them is the `mash-server` crate.
pub mod admin;
pub mod configuration;
pub mod enrollment;
pub mod events;
pub mod status;
pub mod store;

pub use crate::views::scope::Scope as Identity;
