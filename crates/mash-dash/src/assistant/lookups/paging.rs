//! Reading a listing page by page as a stream, for the assistant's lookups
//! that need every page (a period's chats, every memory) rather than one.
use super::{Dash, console, rows};
use serde_json::{Value, json};

/// One page's rows and where the next page starts.
type Page = Result<(Vec<Value>, Option<u64>), String>;

/// The pages of a console listing, in order: `request` from offset 0, then
/// from each `next_offset`. See [`paged`] for when it stops.
pub(super) fn pages<'a>(
    state: &'a Dash,
    name: &'static str,
    request: Value,
    key: &'static str,
    max_offset: u64,
) -> impl futures_util::Stream<Item = Result<Vec<Value>, String>> + 'a {
    paged(max_offset, move |offset| {
        let mut request = request.clone();
        async move {
            request["offset"] = json!(offset);
            let reply = console(state, name, request).await?;
            let next = reply.get("next_offset").and_then(Value::as_u64);
            Ok((rows(&reply, key), next))
        }
    })
}

/// Pages fetched one at a time from offset 0: each `fetch` gives a page's rows
/// and where the next page starts. It ends when there is no next page, the
/// next offset does not move forward (a listing that repeats itself), or the
/// next page would start at `max_offset` or later. A page is fetched only
/// when it is read, so a caller that stops reading asks for nothing more.
fn paged<F, Fut>(
    max_offset: u64,
    fetch: F,
) -> impl futures_util::Stream<Item = Result<Vec<Value>, String>>
where
    F: FnMut(u64) -> Fut,
    Fut: std::future::Future<Output = Page>,
{
    futures_util::stream::try_unfold(
        (Some(0_u64), fetch),
        move |(offset, mut fetch)| async move {
            let Some(offset) = offset.filter(|offset| *offset < max_offset) else {
                return Ok(None);
            };
            let (rows, next) = fetch(offset).await?;
            let next = next.filter(|next| *next > offset);
            Ok(Some((rows, (next, fetch))))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{Page, paged};
    use futures_util::{StreamExt, TryStreamExt};
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};

    /// A listing of `total` rows, `size` per page, that records each offset asked for.
    fn listing(
        total: u64,
        size: u64,
        asked: Arc<Mutex<Vec<u64>>>,
    ) -> impl FnMut(u64) -> std::future::Ready<Page> {
        move |offset| {
            asked.lock().unwrap().push(offset);
            let end = (offset + size).min(total);
            let rows = (offset..end).map(|row| json!(row)).collect();
            std::future::ready(Ok((rows, (end < total).then_some(end))))
        }
    }

    #[tokio::test]
    async fn every_page_is_read_until_the_listing_ends() {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let rows: Vec<Vec<Value>> = paged(1_000, listing(250, 100, asked.clone()))
            .try_collect()
            .await
            .unwrap();
        assert_eq!(
            rows.iter().map(Vec::len).collect::<Vec<_>>(),
            [100, 100, 50]
        );
        assert_eq!(*asked.lock().unwrap(), [0, 100, 200]);
    }

    #[tokio::test]
    async fn it_stops_at_the_cap_and_when_a_listing_repeats_itself() {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let pages = paged(200, listing(1_000, 100, asked.clone())).count().await;
        assert_eq!(pages, 2, "offsets 0 and 100 start before the cap of 200");
        // A next offset that does not move forward ends the listing.
        let stuck = paged(1_000, |_| std::future::ready(Ok((vec![json!(1)], Some(0)))))
            .count()
            .await;
        assert_eq!(stuck, 1);
    }

    #[tokio::test]
    async fn a_reader_that_stops_asks_for_nothing_more() {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let mut pages = Box::pin(paged(1_000, listing(1_000, 100, asked.clone())));
        assert!(pages.next().await.is_some());
        drop(pages);
        assert_eq!(*asked.lock().unwrap(), [0], "only the page that was read");
    }

    #[tokio::test]
    async fn a_failed_page_ends_the_listing_with_its_error() {
        let mut calls = 0;
        let got: Result<Vec<Vec<Value>>, String> = paged(1_000, move |offset| {
            calls += 1;
            std::future::ready(if calls == 1 {
                Ok((vec![json!(offset)], Some(100)))
            } else {
                Err("The lookup failed.".to_owned())
            })
        })
        .try_collect()
        .await;
        assert_eq!(got, Err("The lookup failed.".to_owned()));
    }
}
