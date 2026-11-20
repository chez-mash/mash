//! Conversions between the wire messages and a publication's metadata:
//! its views build, source certificate, start and enrichment evidence.
use super::{
    AppError, Enrichment, EnrichmentOrigin, EnrichmentTimestamps, FirstVisibleMessage,
    MessageBoundary, PublicationStart, SourceCertificate, SourceTimestamp, TimestampOrigin,
    ViewsBuild, enrichment_origin, invalid, protocol, timestamp_origin,
};

impl From<&crate::views::config::Build> for ViewsBuild {
    fn from(build: &crate::views::config::Build) -> Self {
        use crate::views::config::{
            DenseContent, EmbeddingText, Representations, RetrievalUnit, TextPreparation,
        };
        Self {
            retrieval_unit: match build.retrieval_unit {
                RetrievalUnit::Message => 0,
                RetrievalUnit::ParentAndMessage => 1,
            },
            dense_content: match build.dense_content {
                DenseContent::AllVisible => 0,
                DenseContent::UserMessages => 1,
            },
            text_preparation: match build.text_preparation {
                TextPreparation::Preserve => 0,
                TextPreparation::CollapseWhitespace => 1,
            },
            representations: match build.representations {
                Representations::LexicalAndDense => 0,
                Representations::LexicalOnly => 1,
            },
            embedding_text: match build.embedding_text {
                EmbeddingText::Dialogue => 0,
                EmbeddingText::TitleAndDialogue => 1,
            },
            max_embedding_chars: build.max_embedding_chars.map(Into::into),
        }
    }
}

impl TryFrom<ViewsBuild> for crate::views::config::Build {
    type Error = AppError;
    fn try_from(build: ViewsBuild) -> Result<Self, Self::Error> {
        use crate::views::config::{
            DenseContent, EmbeddingText, Representations, RetrievalUnit, TextPreparation,
        };
        Ok(Self {
            retrieval_unit: match build.retrieval_unit {
                0 => RetrievalUnit::Message,
                1 => RetrievalUnit::ParentAndMessage,
                _ => return Err(invalid("invalid retrieval unit")),
            },
            dense_content: match build.dense_content {
                0 => DenseContent::AllVisible,
                1 => DenseContent::UserMessages,
                _ => return Err(invalid("invalid dense content")),
            },
            text_preparation: match build.text_preparation {
                0 => TextPreparation::Preserve,
                1 => TextPreparation::CollapseWhitespace,
                _ => return Err(invalid("invalid text preparation")),
            },
            representations: match build.representations {
                0 => Representations::LexicalAndDense,
                1 => Representations::LexicalOnly,
                _ => return Err(invalid("invalid representations")),
            },
            embedding_text: match build.embedding_text {
                0 => EmbeddingText::Dialogue,
                1 => EmbeddingText::TitleAndDialogue,
                _ => return Err(invalid("invalid embedding text")),
            },
            max_embedding_chars: build
                .max_embedding_chars
                .map(TryInto::try_into)
                .transpose()
                .map_err(|_| invalid("invalid embedding text budget"))?,
        })
    }
}

impl From<&crate::silver::certificate::Certificate> for SourceCertificate {
    fn from(certificate: &crate::silver::certificate::Certificate) -> Self {
        use crate::silver::certificate::Completeness;
        Self {
            revision: certificate.revision,
            interpretation_version: certificate.interpretation_version,
            completeness: match certificate.completeness {
                Completeness::Unverified => 0,
                Completeness::Pending => 1,
                Completeness::Incomplete => 2,
                Completeness::Complete => 3,
            },
            rejected_sources: certificate.rejected_sources,
        }
    }
}

impl TryFrom<SourceCertificate> for crate::silver::certificate::Certificate {
    type Error = AppError;
    fn try_from(certificate: SourceCertificate) -> Result<Self, Self::Error> {
        use crate::silver::certificate::Completeness;
        Ok(Self {
            revision: certificate.revision,
            interpretation_version: certificate.interpretation_version,
            completeness: match certificate.completeness {
                0 => Completeness::Unverified,
                1 => Completeness::Pending,
                2 => Completeness::Incomplete,
                3 => Completeness::Complete,
                _ => return Err(invalid("invalid source completeness")),
            },
            rejected_sources: certificate.rejected_sources,
        })
    }
}

impl From<&protocol::Start> for PublicationStart {
    fn from(start: &protocol::Start) -> Self {
        Self {
            views_build: start.views_build.as_ref().map(ViewsBuild::from),
            source_silver: start.source_silver.as_ref().map(SourceCertificate::from),
            instance: start.instance.clone(),
            revision: start.revision,
            base_revision: start.base_revision,
            snapshot: start.snapshot,
            forgets: start.forgets.clone(),
        }
    }
}

impl TryFrom<PublicationStart> for protocol::Start {
    type Error = AppError;
    fn try_from(start: PublicationStart) -> Result<Self, Self::Error> {
        Ok(Self {
            views_build: start.views_build.map(TryInto::try_into).transpose()?,
            source_silver: start.source_silver.map(TryInto::try_into).transpose()?,
            instance: start.instance,
            revision: start.revision,
            base_revision: start.base_revision,
            snapshot: start.snapshot,
            forgets: start.forgets,
        })
    }
}

impl From<&crate::silver::enrichment::Evidence> for Enrichment {
    fn from(evidence: &crate::silver::enrichment::Evidence) -> Self {
        Self {
            version: evidence.version,
            title: evidence.title.as_ref().map(EnrichmentOrigin::from),
            project: evidence.project.as_ref().map(EnrichmentOrigin::from),
            repository: evidence.repository.as_ref().map(EnrichmentOrigin::from),
            timestamps: evidence
                .timestamps
                .as_ref()
                .map(|times| EnrichmentTimestamps {
                    created_at: times.created_at.as_ref().map(TimestampOrigin::from),
                    updated_at: times.updated_at.as_ref().map(TimestampOrigin::from),
                }),
        }
    }
}

impl TryFrom<Enrichment> for crate::silver::enrichment::Evidence {
    type Error = AppError;
    fn try_from(evidence: Enrichment) -> Result<Self, Self::Error> {
        Ok(Self {
            version: evidence.version,
            title: evidence.title.map(TryInto::try_into).transpose()?,
            project: evidence.project.map(TryInto::try_into).transpose()?,
            repository: evidence.repository.map(TryInto::try_into).transpose()?,
            timestamps: evidence
                .timestamps
                .map(|times| {
                    Ok(crate::silver::enrichment::Timestamps {
                        created_at: times.created_at.map(TryInto::try_into).transpose()?,
                        updated_at: times.updated_at.map(TryInto::try_into).transpose()?,
                    })
                })
                .transpose()?,
        })
    }
}

impl From<&crate::silver::enrichment::Origin> for EnrichmentOrigin {
    fn from(origin: &crate::silver::enrichment::Origin) -> Self {
        use crate::silver::enrichment::{Origin, SourceField};
        let kind = match origin {
            Origin::Unknown => enrichment_origin::Kind::Unknown(true),
            Origin::SourceMetadata { field } => enrichment_origin::Kind::SourceField(
                match field {
                    SourceField::Cwd => "cwd",
                    SourceField::SessionTitle => "session.title",
                    SourceField::ThreadTitle => "threads.title",
                    SourceField::SessionDirectory => "session.directory",
                    SourceField::ProjectWorktree => "project.worktree",
                }
                .to_owned(),
            ),
            Origin::FirstVisibleUserMessage {
                message_id,
                rule_version,
            } => enrichment_origin::Kind::FirstVisible(FirstVisibleMessage {
                message_id: message_id.clone(),
                rule_version: *rule_version,
            }),
        };
        Self { kind: Some(kind) }
    }
}

impl TryFrom<EnrichmentOrigin> for crate::silver::enrichment::Origin {
    type Error = AppError;
    fn try_from(origin: EnrichmentOrigin) -> Result<Self, Self::Error> {
        use crate::silver::enrichment::SourceField;
        match origin
            .kind
            .ok_or_else(|| invalid("missing enrichment origin"))?
        {
            enrichment_origin::Kind::Unknown(true) => Ok(Self::Unknown),
            enrichment_origin::Kind::SourceField(field) => Ok(Self::SourceMetadata {
                field: match field.as_str() {
                    "cwd" => SourceField::Cwd,
                    "session.title" => SourceField::SessionTitle,
                    "threads.title" => SourceField::ThreadTitle,
                    "session.directory" => SourceField::SessionDirectory,
                    "project.worktree" => SourceField::ProjectWorktree,
                    _ => return Err(invalid("invalid enrichment source field")),
                },
            }),
            enrichment_origin::Kind::FirstVisible(message) => Ok(Self::FirstVisibleUserMessage {
                message_id: message.message_id,
                rule_version: message.rule_version,
            }),
            enrichment_origin::Kind::Unknown(_) => Err(invalid("invalid enrichment origin")),
        }
    }
}

impl From<&crate::silver::enrichment::TimestampOrigin> for TimestampOrigin {
    fn from(origin: &crate::silver::enrichment::TimestampOrigin) -> Self {
        use crate::silver::enrichment::{Boundary, TimestampField, TimestampOrigin as Origin};
        let kind = match origin {
            Origin::Unknown => timestamp_origin::Kind::Unknown(true),
            Origin::SourceMetadata {
                field,
                rule_version,
            } => timestamp_origin::Kind::Source(SourceTimestamp {
                field: match field {
                    TimestampField::SessionTimeCreated => "session.time_created",
                    TimestampField::SessionTimeUpdated => "session.time_updated",
                    TimestampField::ThreadCreatedAtMs => "threads.created_at_ms",
                    TimestampField::ThreadUpdatedAtMs => "threads.updated_at_ms",
                    TimestampField::ThreadCreatedAt => "threads.created_at",
                    TimestampField::ThreadUpdatedAt => "threads.updated_at",
                }
                .to_owned(),
                rule_version: *rule_version,
            }),
            Origin::MessageBoundary {
                message_id,
                selection,
                rule_version,
            } => timestamp_origin::Kind::Boundary(MessageBoundary {
                message_id: message_id.clone(),
                maximum: *selection == Boundary::Maximum,
                rule_version: *rule_version,
            }),
        };
        Self { kind: Some(kind) }
    }
}

impl TryFrom<TimestampOrigin> for crate::silver::enrichment::TimestampOrigin {
    type Error = AppError;
    fn try_from(origin: TimestampOrigin) -> Result<Self, Self::Error> {
        use crate::silver::enrichment::{Boundary, TimestampField};
        match origin
            .kind
            .ok_or_else(|| invalid("missing timestamp origin"))?
        {
            timestamp_origin::Kind::Unknown(true) => Ok(Self::Unknown),
            timestamp_origin::Kind::Source(source) => Ok(Self::SourceMetadata {
                field: match source.field.as_str() {
                    "session.time_created" => TimestampField::SessionTimeCreated,
                    "session.time_updated" => TimestampField::SessionTimeUpdated,
                    "threads.created_at_ms" => TimestampField::ThreadCreatedAtMs,
                    "threads.updated_at_ms" => TimestampField::ThreadUpdatedAtMs,
                    "threads.created_at" => TimestampField::ThreadCreatedAt,
                    "threads.updated_at" => TimestampField::ThreadUpdatedAt,
                    _ => return Err(invalid("invalid timestamp source field")),
                },
                rule_version: source.rule_version,
            }),
            timestamp_origin::Kind::Boundary(boundary) => Ok(Self::MessageBoundary {
                message_id: boundary.message_id,
                selection: if boundary.maximum {
                    Boundary::Maximum
                } else {
                    Boundary::Minimum
                },
                rule_version: boundary.rule_version,
            }),
            timestamp_origin::Kind::Unknown(_) => Err(invalid("invalid timestamp origin")),
        }
    }
}
