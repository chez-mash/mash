use super::SearchUnit;
use crate::AppError;

/// One page, `[offset, offset + limit)`, of a search's final ranked list.
/// Ranking never depends on the page, so pages partition one ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub offset: usize,
    pub limit: usize,
}

impl Page {
    /// Requests share the `views.query.limit` configuration bound.
    pub const MAX_LIMIT: usize = 200;

    pub fn new(limit: usize, offset: usize, unit: SearchUnit) -> Result<Self, AppError> {
        let window = unit.result_window();
        if !(1..=Self::MAX_LIMIT).contains(&limit) || offset >= window {
            return Err(AppError::usage(format!(
                "search pages require limit 1–{} and offset 0–{} for unit {}",
                Self::MAX_LIMIT,
                window - 1,
                unit.name()
            )));
        }
        Ok(Self { offset, limit })
    }

    /// The first page of `limit` results, for callers outside request validation.
    #[cfg(test)]
    pub const fn first(limit: usize) -> Self {
        Self { offset: 0, limit }
    }

    /// Slice the ranked results; `next_offset` is set only when more remain.
    pub(super) fn take<T>(
        self,
        ranked: impl Iterator<Item = T>,
        window: usize,
    ) -> (Vec<T>, Option<usize>) {
        let mut ranked = ranked.take(window).skip(self.offset);
        let page = ranked.by_ref().take(self.limit).collect();
        let next = self.offset + self.limit;
        (page, ranked.next().is_some().then_some(next))
    }
}
