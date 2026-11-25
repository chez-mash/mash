//! Each kind of record's own lifecycle and fields: a decision, a preference
//! or a note. A record's body holds exactly one of these details.
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::conflict;
use crate::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DecisionState {
    Proposed,
    Accepted,
    Superseded,
    Withdrawn,
}
/// Who made a decision. An agent may settle a choice of its own, such as how
/// to structure code, by citing the message where it made and explained it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DecidedBy {
    #[default]
    User,
    Agent,
}

impl DecidedBy {
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde passes the field by reference"
    )]
    fn is_user(&self) -> bool {
        *self == Self::User
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceState {
    Active,
    Superseded,
    Withdrawn,
}

/// A note is something the person asked to keep that is neither a decision
/// nor a preference, such as where a service runs or when a job resets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NoteState {
    Active,
    Superseded,
    Withdrawn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Details {
    Decision {
        state: DecisionState,
        /// Omit for the person's decision; `agent` for a choice the agent made.
        #[serde(default, skip_serializing_if = "DecidedBy::is_user")]
        decided_by: DecidedBy,
        #[serde(default)]
        alternatives: Vec<String>,
        #[serde(default)]
        consequences: Vec<String>,
    },
    Preference {
        state: PreferenceState,
        #[serde(default)]
        conditions: Vec<String>,
    },
    /// Saved only when the person asks, so it is always established.
    Note { state: NoteState },
}

impl Details {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Decision { .. } => "decision",
            Self::Preference { .. } => "preference",
            Self::Note { .. } => "note",
        }
    }

    pub const fn current(&self) -> bool {
        matches!(
            self,
            Self::Decision {
                state: DecisionState::Accepted,
                ..
            } | Self::Preference {
                state: PreferenceState::Active,
                ..
            } | Self::Note {
                state: NoteState::Active
            }
        )
    }

    pub fn supersede(&mut self) -> Result<(), AppError> {
        if !self.current() {
            return Err(conflict("only current records can be superseded"));
        }
        match self {
            Self::Decision { state, .. } => *state = DecisionState::Superseded,
            Self::Preference { state, .. } => *state = PreferenceState::Superseded,
            Self::Note { state } => *state = NoteState::Superseded,
        }
        Ok(())
    }

    /// A choice the agent made itself, which the person has not endorsed.
    pub const fn agent_decided(&self) -> bool {
        matches!(
            self,
            Self::Decision {
                decided_by: DecidedBy::Agent,
                ..
            }
        )
    }

    pub const fn superseded(&self) -> bool {
        matches!(
            self,
            Self::Decision {
                state: DecisionState::Superseded,
                ..
            } | Self::Preference {
                state: PreferenceState::Superseded,
                ..
            } | Self::Note {
                state: NoteState::Superseded
            }
        )
    }
}
