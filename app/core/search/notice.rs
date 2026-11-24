//! What an empty search page says about why it may be empty.
use super::SearchResponse;
use crate::config::ResolvedConfig;

pub(super) fn empty_notice(
    search: &SearchResponse,
    configuration: &ResolvedConfig,
    local: bool,
) -> Option<String> {
    let empty = match &search.results {
        crate::search_results::Results::Chats(chats) => chats.is_empty(),
        crate::search_results::Results::Messages(messages) => messages.is_empty(),
    };
    if !empty {
        return None;
    }
    let refresh = "For a recent chat on this computer, run `mash index` and search again.";
    if search.intent != crate::views::exact::Intent::ExactMention {
        return local.then(|| refresh.to_owned());
    }
    // The query itself stays out of the notice: it is redacted separately.
    let query = search.query.trim();
    let words = crate::views::matching::tokens(query).len();
    let mut notice = if words > 1 && crate::views::exact::quoted(query) {
        "No indexed match for this phrase. Remove the quotes to match the words in any order."
    } else if words > 1 {
        "No indexed message contains all of these words. Try fewer words or the most distinctive one."
    } else {
        "No indexed match for this text. Word search checks message text. Use `--project` for paths or `mash list-chats` for chat IDs."
    }
    .to_owned();
    if local {
        notice.push(' ');
        notice.push_str(refresh);
        if !configuration.views.build.dense_enabled() {
            notice.push_str(" To search by meaning, run `mash models install` and follow https://getmash.io/indexing/#enable-semantic-search");
        }
    }
    Some(notice)
}
