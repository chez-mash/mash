//! The caching and browser-safety headers every dashboard response carries.
use axum::http::{HeaderValue, header};
use axum::response::Response;

/// How long a browser may keep a response.
#[derive(Clone, Copy)]
pub(super) enum Cache {
    Api,
    Hashed,
    Image,
    Page,
}

impl Cache {
    pub(super) fn of(path: &str) -> Self {
        if path.starts_with("/api/") {
            Self::Api
        } else if path.starts_with("/assets/") {
            Self::Hashed
        } else if std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("svg") || extension.eq_ignore_ascii_case("png")
            })
        {
            Self::Image
        } else {
            Self::Page
        }
    }
}

/// The caching and browser-safety headers every response carries.
pub(super) fn harden(mut response: Response, cache: Cache) -> Response {
    // An attachment chose its own caching (by digest) and a stricter policy.
    let chosen = response
        .extensions_mut()
        .remove::<super::attachment::Chosen>();
    let headers = response.headers_mut();
    // Hashed build assets may be kept for good; images keep their names across
    // releases, so they are rechecked like the page.
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(match cache {
            Cache::Api => "no-store",
            Cache::Hashed => "public, max-age=31536000, immutable",
            Cache::Image | Cache::Page => "no-cache",
        }),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'",
        ),
    );
    for (name, value) in chosen.into_iter().flat_map(|chosen| chosen.0) {
        headers.insert(name, value);
    }
    response
}
