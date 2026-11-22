//! mash:// links: the address of a page in the dashboard that any program can
//! open in the Mash desktop app.
//!
//! A link is the dashboard's own route with `mash://` in place of `#/`:
//! `mash://chat/abc?node=laptop&message=m-2` opens `#/chat/abc?node=…`, and
//! `mash://` alone opens the overview. The dashboard's router decides what a
//! route means and shows its "page not found" page for anything it does not
//! know, so this module only checks that a link is well formed and safe to
//! put in the page's address: it never adds pages of its own.

/// The longest link accepted. Real routes are far shorter.
const MAX_LINK: usize = 4096;

/// The dashboard route (`#/…`) a mash:// link opens, or `None` when it is not
/// a mash:// link or not safe to follow.
pub fn route(link: &str) -> Option<String> {
    let link = link.trim();
    if link.len() > MAX_LINK {
        return None;
    }
    let (scheme, rest) = link.split_once(':')?;
    if !scheme.eq_ignore_ascii_case("mash") {
        return None;
    }
    // mash://chat/abc, mash:/chat/abc and mash:///chat/abc all name the same page.
    let rest = rest.trim_start_matches('/');
    // A fragment would only fight the dashboard's own one; drop it.
    let rest = rest.split('#').next().unwrap_or_default();
    if !rest.bytes().all(allowed) {
        return None;
    }
    let (path, query) = match rest.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (rest, None),
    };
    if path.split('/').any(|segment| segment == "." || segment == "..") {
        return None;
    }
    let path = path.trim_end_matches('/');
    Some(match query {
        Some(query) if !query.is_empty() => format!("#/{path}?{query}"),
        _ => format!("#/{path}"),
    })
}

/// Characters a link may carry: letters, digits, the unreserved marks, and the
/// delimiters a route uses. Anything else must arrive percent-encoded, so
/// spaces, quotes, angle brackets and control characters are refused.
fn allowed(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"-._~%/?&=+,:@!$'()*;".contains(&byte)
}

#[cfg(test)]
mod tests {
    use super::route;

    #[test]
    fn a_link_is_the_dashboard_route_behind_mash() {
        assert_eq!(
            route("mash://chat/abc?node=laptop&message=m-2").as_deref(),
            Some("#/chat/abc?node=laptop&message=m-2")
        );
        assert_eq!(
            route("mash://record/record-sign-in-tokens?node=laptop").as_deref(),
            Some("#/record/record-sign-in-tokens?node=laptop")
        );
        assert_eq!(route("mash://search?q=checkout%20outage").as_deref(), Some("#/search?q=checkout%20outage"));
        assert_eq!(route("mash://records?kind=decision").as_deref(), Some("#/records?kind=decision"));
    }

    #[test]
    fn bare_and_differently_slashed_links_name_the_same_page() {
        assert_eq!(route("mash://").as_deref(), Some("#/"));
        assert_eq!(route("mash:").as_deref(), Some("#/"));
        assert_eq!(route("MASH://chat/abc").as_deref(), Some("#/chat/abc"));
        assert_eq!(route("mash:/chat/abc").as_deref(), Some("#/chat/abc"));
        assert_eq!(route("mash:///chat/abc/").as_deref(), Some("#/chat/abc"));
        assert_eq!(route("  mash://chat/abc\n").as_deref(), Some("#/chat/abc"));
        assert_eq!(route("mash://chat/abc?").as_deref(), Some("#/chat/abc"));
        assert_eq!(route("mash://chat/abc#top").as_deref(), Some("#/chat/abc"));
    }

    #[test]
    fn identifiers_keep_their_percent_encoding() {
        assert_eq!(
            route("mash://chat/a%2Fb?node=build%20box").as_deref(),
            Some("#/chat/a%2Fb?node=build%20box")
        );
    }

    #[test]
    fn other_schemes_and_unsafe_links_are_refused() {
        for link in [
            "https://getmash.io/",
            "javascript:alert(1)",
            "mashy://chat/abc",
            "file:///etc/passwd",
            "mash://chat/<script>",
            "mash://chat/a b",
            "mash://chat/\"x\"",
            "mash://chat/abc\r\nInjected: 1",
            "mash://chat/../../settings",
            "mash://./chat",
        ] {
            assert_eq!(route(link), None, "{link:?}");
        }
        assert_eq!(route(&format!("mash://chat/{}", "a".repeat(5000))), None);
    }
}
