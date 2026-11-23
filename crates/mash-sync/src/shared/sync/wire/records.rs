//! Conversions between the wire messages and Mash's history records:
//! chats, messages, tombstones and what they carry.
use super::{
    Amend, AppError, Attachment, Chat, Enrichment, MessageContext, NativeMessage, NativeRecord,
    Tombstone, ToolActivity, ToolCall, ToolResult, Vector, invalid, native_record, protocol,
};

impl From<&protocol::ChatRecord> for NativeRecord {
    fn from(record: &protocol::ChatRecord) -> Self {
        Self {
            body: Some(native_record::Body::Chat(Chat {
                enrichment: record.enrichment.as_ref().map(Enrichment::from),
                id: record.id.clone(),
                provider: record.provider.as_str().to_owned(),
                title: record.title.clone(),
                project: record.project.clone(),
                repository: record.repository.clone(),
                created_at: record.created_at,
                updated_at: record.updated_at,
                subagent: record.subagent.as_ref().map(ChatSubagent::from),
            })),
        }
    }
}

/// A chat's subagent marker; see `protocol::ChatRecord::subagent`. Every
/// field is optional, as the agent may record any of them or none.
#[derive(Clone, PartialEq, prost::Message)]
pub struct ChatSubagent {
    #[prost(string, optional, tag = "1")]
    pub parent_chat_id: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub label: Option<String>,
    #[prost(string, optional, tag = "3")]
    pub role: Option<String>,
    #[prost(uint32, optional, tag = "4")]
    pub depth: Option<u32>,
    #[prost(bool, optional, tag = "5")]
    pub finished: Option<bool>,
    #[prost(string, optional, tag = "6")]
    pub spawn_call_id: Option<String>,
}

impl From<&crate::silver::subagent::Subagent> for ChatSubagent {
    fn from(marker: &crate::silver::subagent::Subagent) -> Self {
        Self {
            parent_chat_id: marker.parent_chat_id.clone(),
            label: marker.label.clone(),
            role: marker.role.clone(),
            depth: marker.depth,
            finished: marker.finished,
            spawn_call_id: marker.spawn_call_id.clone(),
        }
    }
}

impl From<ChatSubagent> for crate::silver::subagent::Subagent {
    fn from(marker: ChatSubagent) -> Self {
        Self {
            parent_chat_id: marker.parent_chat_id,
            label: marker.label,
            role: marker.role,
            depth: marker.depth,
            finished: marker.finished,
            spawn_call_id: marker.spawn_call_id,
        }
    }
}

/// Baseline membership digest for a chat record. It leaves out the subagent
/// marker: a server older than the marker drops it on decoding and digests
/// what it kept, and both sides must agree for a baseline to complete.
pub fn member_digest(record: &protocol::ChatRecord) -> Result<String, AppError> {
    use prost::Message as _;
    let mut record = NativeRecord::from(record);
    if let Some(native_record::Body::Chat(chat)) = &mut record.body {
        chat.subagent = None;
    }
    let mut bytes = Vec::new();
    record
        .encode_length_delimited(&mut bytes)
        .map_err(|_| AppError::internal("encode backfill member"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Baseline membership digest for a tombstone, independent of the changed
/// chat record that may arrive later.
pub fn tombstone_member_digest(chat_id: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mash-backfill-tombstone-v1\0");
    hasher.update(chat_id.as_bytes());
    hasher.finalize().to_hex().to_string()
}

impl From<&protocol::Amendment> for NativeRecord {
    fn from(amendment: &protocol::Amendment) -> Self {
        let chat = match NativeRecord::from(&amendment.chat).body {
            Some(native_record::Body::Chat(chat)) => Some(chat),
            _ => None,
        };
        Self {
            body: Some(native_record::Body::Amend(Amend {
                chat,
                replaced: amendment.replaced.clone(),
                messages: amendment.messages,
            })),
        }
    }
}

impl TryFrom<Chat> for protocol::ChatRecord {
    type Error = AppError;
    fn try_from(record: Chat) -> Result<Self, Self::Error> {
        Ok(Self {
            enrichment: record.enrichment.map(TryInto::try_into).transpose()?,
            id: record.id,
            provider: record.provider.parse()?,
            title: record.title,
            project: record.project,
            repository: record.repository,
            created_at: record.created_at,
            updated_at: record.updated_at,
            subagent: record.subagent.map(Into::into),
        })
    }
}

impl TryFrom<&protocol::MessageRecord> for NativeRecord {
    type Error = AppError;
    fn try_from(record: &protocol::MessageRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            body: Some(native_record::Body::Message(NativeMessage {
                attachments: record.attachments.iter().map(Attachment::from).collect(),
                id: record.id.clone(),
                chat_id: record.chat_id.clone(),
                ordinal: record.ordinal,
                context: Some(MessageContext::from(&record.context)),
                role: record.role.as_str().to_owned(),
                content: record.content.clone(),
                projection: record.projection.clone(),
                created_at: record.created_at,
                generation: record.generation.clone(),
                vector: record.vector.as_ref().map(|values| Vector {
                    values: values.clone(),
                }),
                tools: Some(ToolActivity::try_from(&record.tools)?),
            })),
        })
    }
}

impl TryFrom<NativeMessage> for protocol::MessageRecord {
    type Error = AppError;
    fn try_from(record: NativeMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            attachments: record
                .attachments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            id: record.id,
            chat_id: record.chat_id,
            ordinal: record.ordinal,
            context: record
                .context
                .ok_or_else(|| invalid("missing message context"))?
                .try_into()?,
            role: record.role.into(),
            content: record.content,
            projection: record.projection,
            created_at: record.created_at,
            generation: record.generation,
            vector: record.vector.map(|vector| vector.values),
            tools: record
                .tools
                .map_or_else(|| Ok(crate::tools::Activity::default()), TryInto::try_into)?,
        })
    }
}

impl From<&protocol::Tombstone> for NativeRecord {
    fn from(record: &protocol::Tombstone) -> Self {
        Self {
            body: Some(native_record::Body::Tombstone(Tombstone {
                provider: record.provider.as_str().to_owned(),
                chat_id: record.chat_id.clone(),
            })),
        }
    }
}

impl TryFrom<Tombstone> for protocol::Tombstone {
    type Error = AppError;
    fn try_from(record: Tombstone) -> Result<Self, Self::Error> {
        Ok(Self {
            provider: record.provider.parse()?,
            chat_id: record.chat_id,
        })
    }
}

impl From<&crate::message_context::MessageContext> for MessageContext {
    fn from(context: &crate::message_context::MessageContext) -> Self {
        match context {
            crate::message_context::MessageContext::Linear => Self {
                kind: 0,
                parent_id: None,
            },
            crate::message_context::MessageContext::Root => Self {
                kind: 1,
                parent_id: None,
            },
            crate::message_context::MessageContext::Parent { id } => Self {
                kind: 2,
                parent_id: Some(id.clone()),
            },
        }
    }
}

impl TryFrom<MessageContext> for crate::message_context::MessageContext {
    type Error = AppError;
    fn try_from(context: MessageContext) -> Result<Self, Self::Error> {
        match (context.kind, context.parent_id) {
            (0, None) => Ok(Self::Linear),
            (1, None) => Ok(Self::Root),
            (2, Some(id)) if !id.is_empty() => Ok(Self::Parent { id }),
            _ => Err(invalid("invalid message context")),
        }
    }
}

impl From<&crate::attachments::Metadata> for Attachment {
    fn from(record: &crate::attachments::Metadata) -> Self {
        use crate::attachments::Availability;
        Self {
            id: record.id.clone(),
            name: record.name.clone(),
            media_type: record.media_type.clone(),
            availability: match record.availability {
                Availability::NotUploaded => 0,
                Availability::Retained => 1,
                Availability::Missing => 2,
                Availability::NotAllowed => 3,
                Availability::TooLarge => 4,
                Availability::Invalid => 5,
                Availability::External => 6,
            },
            bytes: record.bytes,
            digest: record.digest.clone(),
            redacted: record.redacted,
        }
    }
}

impl TryFrom<Attachment> for crate::attachments::Metadata {
    type Error = AppError;
    fn try_from(record: Attachment) -> Result<Self, Self::Error> {
        use crate::attachments::Availability;
        Ok(Self {
            id: record.id,
            name: record.name,
            media_type: record.media_type,
            availability: match record.availability {
                0 => Availability::NotUploaded,
                1 => Availability::Retained,
                2 => Availability::Missing,
                3 => Availability::NotAllowed,
                4 => Availability::TooLarge,
                5 => Availability::Invalid,
                6 => Availability::External,
                _ => return Err(invalid("invalid attachment availability")),
            },
            bytes: record.bytes,
            digest: record.digest,
            redacted: record.redacted,
        })
    }
}

impl TryFrom<&crate::tools::Activity> for ToolActivity {
    type Error = AppError;
    fn try_from(activity: &crate::tools::Activity) -> Result<Self, Self::Error> {
        Ok(Self {
            calls: activity
                .tool_calls
                .iter()
                .map(|call| {
                    Ok(ToolCall {
                        name: call.name.clone(),
                        call_id: call.call_id.clone(),
                        input_json: call
                            .input
                            .as_ref()
                            .map(serde_json::to_string)
                            .transpose()
                            .map_err(|_| AppError::internal("encode opaque tool input"))?,
                    })
                })
                .collect::<Result<_, AppError>>()?,
            results: activity
                .tool_results
                .iter()
                .map(|result| ToolResult {
                    call_id: result.call_id.clone(),
                    is_error: result.is_error,
                })
                .collect(),
        })
    }
}

impl TryFrom<ToolActivity> for crate::tools::Activity {
    type Error = AppError;
    fn try_from(activity: ToolActivity) -> Result<Self, Self::Error> {
        Ok(Self {
            tool_calls: activity
                .calls
                .into_iter()
                .map(|call| {
                    Ok(crate::tools::ToolCall {
                        name: call.name,
                        call_id: call.call_id,
                        input: call
                            .input_json
                            .map(|json| {
                                serde_json::from_str(&json)
                                    .map_err(|_| invalid("invalid opaque tool JSON"))
                            })
                            .transpose()?,
                    })
                })
                .collect::<Result<_, AppError>>()?,
            tool_results: activity
                .results
                .into_iter()
                .map(|result| crate::tools::ToolResult {
                    call_id: result.call_id,
                    is_error: result.is_error,
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod subagent_tests;
