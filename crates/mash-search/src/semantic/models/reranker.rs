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

    /// Waits for a load in flight and reports whether loading failed. A
    /// settled reranker has no thread left inside the model runtime.
    pub(super) fn settle(&mut self) -> bool {
        if self.loading.is_some() {
            // A failure is recorded in `failed`; the next search that needs
            // the model loads it again and reports the reason.
            let _ = self.get();
        }
        self.failed
    }
}

// The loading thread must never outlive its owner. ONNX Runtime is linked
// into the executable, so process exit destroys its C++ registries while any
// thread still creating a session reads them: a segmentation fault after the
// work has already succeeded. Session creation cannot be cancelled, so a
// search that ends before it reranks waits here for the load it started.
impl Drop for Lazy {
    fn drop(&mut self) {
        if let Some(loading) = self.loading.take() {
            let _ = loading.join();
        }
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
