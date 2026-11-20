//! What a durable transfer carries, as the catalog stores it.
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

/// The kind of a durable transfer. Stored as text; a value this release does
/// not know, from a newer one sharing the catalog, is kept as it was read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A batch of history.
    Publication,
    /// A progressive backfill's manifest.
    BackfillManifest,
    /// One attachment's bytes.
    Attachment,
    Unknown(String),
}

impl Kind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Publication => "publication",
            Self::BackfillManifest => "backfill_manifest",
            Self::Attachment => "attachment",
            Self::Unknown(other) => other,
        }
    }

    pub fn parse(text: &str) -> Self {
        match text {
            "publication" => Self::Publication,
            "backfill_manifest" => Self::BackfillManifest,
            "attachment" => Self::Attachment,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl ToSql for Kind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(ValueRef::Text(
            self.as_str().as_bytes(),
        )))
    }
}

impl FromSql for Kind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(Self::parse)
    }
}

#[cfg(test)]
mod tests {
    use super::Kind;

    #[test]
    fn every_kind_keeps_its_stored_text_and_an_unknown_one_survives() {
        for text in [
            "publication",
            "backfill_manifest",
            "attachment",
            "future_kind",
        ] {
            assert_eq!(Kind::parse(text).as_str(), text);
        }
        assert_eq!(Kind::parse("attachment"), Kind::Attachment);
        assert_eq!(Kind::parse("later"), Kind::Unknown("later".into()));
    }
}
