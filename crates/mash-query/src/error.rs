//! Query policy failures independent of CLI/MCP/HTTP error envelopes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidQuery,
    CommonWords,
    UnsupportedScript,
    UnsupportedLanguage(&'static str),
}

const WORD_SEARCH_GUIDANCE: &str =
    "For other languages, use word search (exact-mention) with words from the original chat.";

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidQuery => formatter.write_str(
                "search query must contain letters or numbers and be at most 16384 bytes",
            ),
            Self::CommonWords => formatter.write_str(
                "enter a more specific search query; common words alone do not identify a topic",
            ),
            Self::UnsupportedScript => write!(
                formatter,
                "Search by meaning uses English queries. {WORD_SEARCH_GUIDANCE}"
            ),
            Self::UnsupportedLanguage(language) => write!(
                formatter,
                "Search by meaning uses English queries. The query appears to be in {language}. {WORD_SEARCH_GUIDANCE}"
            ),
        }
    }
}

impl std::error::Error for Error {}
