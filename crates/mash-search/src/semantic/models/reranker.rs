//! CPU contextual reranking in batches the caller bounds.
use super::{AppError, Path, TextRerank, load_reranker};

pub(super) struct Reranker {
    pub(super) cpu: TextRerank,
}

/// A reranker that is loaded, loading on its own thread, or not yet asked
/// for. A search needs the embedding model first and the reranker last, so
/// retrieval runs while the larger model loads. A search with a reranking
/// worker needs it only if the worker fails, and then loads it on demand.
pub(super) struct Lazy {
    root: std::path::PathBuf,
    loading: Option<std::thread::JoinHandle<Result<Reranker, AppError>>>,
    loaded: Option<Reranker>,
    failed: bool,
}

impl Lazy {
    /// Start loading now, in the background.
    pub(super) fn start(root: &Path) -> Self {
        let mut lazy = Self::deferred(root);
        let root = lazy.root.clone();
        lazy.loading = Some(std::thread::spawn(move || Reranker::load(&root)));
        lazy
    }

    /// Load only when first needed.
    pub(super) fn deferred(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            loading: None,
            loaded: None,
            failed: false,
        }
    }

    /// Waits for the load, or performs it. A failed load is reported by the
    /// search that first needed the model; `failed` then keeps this backend
    /// out of the cache.
    pub(super) fn get(&mut self) -> Result<&mut Reranker, AppError> {
        if self.loaded.is_none() {
            let loaded = match self.loading.take() {
                Some(loading) => loading
                    .join()
                    .unwrap_or_else(|_| Err(AppError::internal("reranker loading failed"))),
                None => Reranker::load(&self.root),
            };
            self.failed = loaded.is_err();
            self.loaded = Some(loaded?);
        }
        self.loaded
            .as_mut()
            .ok_or_else(|| AppError::internal("the reranking model is not loaded"))
    }

    pub(super) const fn failed(&self) -> bool {
        self.failed
    }
}

#[cfg(test)]
mod tests;

impl Reranker {
    pub(super) fn load(root: &Path) -> Result<Self, AppError> {
        Ok(Self {
            cpu: load_reranker(root)?,
        })
    }

    pub(super) fn run(
        &mut self,
        query: &str,
        documents: &[&str],
    ) -> Result<Vec<fastembed::RerankResult>, AppError> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        self.cpu
            // The caller already sized this batch.
            .rerank(query, documents, false, Some(documents.len()))
            .map_err(|error| AppError::model(format!("reranking inference failed: {error}")))
    }
}
