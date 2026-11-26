//! Explicit model installation, asset validation, and cached search backends.
use super::embeddings::EmbeddingPool;
use super::{
    EMBEDDING_MODEL, EMBEDDING_REPOSITORY, MARKER_NAME, RERANKER_MODEL, RERANKER_REPOSITORY,
};
use crate::AppError;
use fastembed::{
    InitOptionsUserDefined, RerankInitOptionsUserDefined, TextEmbedding, TextRerank,
    UserDefinedEmbeddingModel, UserDefinedRerankingModel,
};
use mash_model::file_lock::FileLock;
use serde::Serialize;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;
mod assets;
mod downloads;
mod generations;
pub mod indexer;
mod install;
mod reranker;
use assets::InstallMarker;
pub(super) use assets::{read_model_file, read_tokenizer_files, snapshot};

pub struct Models {
    root: PathBuf,
    reranking: crate::config::Reranking,
    accelerated_embedding: bool,
}

/// The configured remote reranker, or why this search cannot use it.
type RemoteReranker = Option<
    Result<
        std::sync::Arc<super::remote_reranker::Client>,
        crate::observability::retrieval::RerankerFallback,
    >,
>;

pub struct Backend {
    embedding: TextEmbedding,
    reranker: Option<reranker::Lazy>,
    // Resolved on every load: configuration and credentials can change while
    // a cached backend is idle.
    remote_reranker: RemoteReranker,
    // Retained with cached models and lazy Core ML sessions until their drop.
    assets_lease: Option<FileLock>,
}

#[derive(PartialEq, Eq)]
struct BackendCacheKey {
    ranking: crate::views::config::Ranking,
    root: PathBuf,
    marker_modified: Option<SystemTime>,
    files: Vec<(String, u64, Option<SystemTime>)>,
}

struct CachedBackend {
    key: BackendCacheKey,
    backend: Backend,
}

// MCP already serializes model work. Retain at most one idle local backend;
// in-flight searches own their backend without holding a lock across I/O.
static LOCAL_BACKEND: Mutex<Option<CachedBackend>> = Mutex::new(None);

/// Passages scored per inference call. Eight per call measured about 5%
/// faster than four on a 16-core server and about 10% slower on an M4, so
/// this stays at four.
const RERANK_BATCH: usize = 4;

pub struct LoadedBackend {
    backend: Option<Backend>,
    cache_key: Option<BackendCacheKey>,
}

impl Deref for LoadedBackend {
    type Target = Backend;

    fn deref(&self) -> &Backend {
        self.backend
            .as_ref()
            .expect("loaded backend is present until drop")
    }
}

impl DerefMut for LoadedBackend {
    fn deref_mut(&mut self) -> &mut Backend {
        self.backend
            .as_mut()
            .expect("loaded backend is present until drop")
    }
}

impl Drop for LoadedBackend {
    fn drop(&mut self) {
        if let (Some(key), Some(mut backend)) = (self.cache_key.take(), self.backend.take()) {
            // Cache no load in flight (it would race process exit) and no
            // failed model: the next search loads it and reports the reason.
            if backend
                .reranker
                .as_mut()
                .is_some_and(reranker::Lazy::settle)
            {
                return;
            }
            let previous = LOCAL_BACKEND
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .replace(CachedBackend { key, backend });
            // Model destruction can be expensive; do it after releasing the lock.
            drop(previous);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InstallSummary {
    pub embedding_model: &'static str,
    pub reranker_model: &'static str,
    pub embedding_dimensions: usize,
    pub files: usize,
}

impl Models {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            reranking: crate::config::Reranking::default(),
            accelerated_embedding: false,
        }
    }

    /// Whether the indexer loaded by this handle tries the GPU first.
    pub const fn with_accelerated_embedding(mut self, accelerated: bool) -> Self {
        self.accelerated_embedding = accelerated;
        self
    }

    /// Contextual reranking placement for searches loaded by this handle.
    pub fn with_reranking(mut self, reranking: crate::config::Reranking) -> Self {
        self.reranking = reranking;
        self
    }

    #[cfg(test)]
    pub fn require_indexer(&self) -> Result<(), AppError> {
        self.require_installed()
    }

    pub fn is_installed(&self) -> bool {
        self.read_valid_marker().is_ok()
    }

    pub fn require_installed(&self) -> Result<(), AppError> {
        if !self.root.join(MARKER_NAME).is_file() {
            return Err(AppError::models_missing());
        }
        self.read_valid_marker().map(|_| ())
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn load(&self) -> Result<Option<LoadedBackend>, AppError> {
        self.load_search(crate::views::config::Ranking::default())
    }

    #[tracing::instrument(name = "load_search_models", skip_all)]
    pub fn load_search(
        &self,
        ranking: crate::views::config::Ranking,
    ) -> Result<Option<LoadedBackend>, AppError> {
        if !self.root.join(MARKER_NAME).is_file() {
            return Ok(None);
        }
        let selected = generations::select(&self.root)?;
        let marker = selected.marker;
        let asset_root = selected.root;
        let key = BackendCacheKey {
            ranking,
            root: asset_root.clone(),
            marker_modified: fs::metadata(self.root.join(MARKER_NAME))
                .map_err(AppError::io)?
                .modified()
                .ok(),
            files: marker
                .files
                .into_iter()
                .map(|file| {
                    let modified = fs::metadata(asset_root.join(&file.path))
                        .map_err(AppError::io)?
                        .modified()
                        .ok();
                    Ok((file.path, file.size, modified))
                })
                .collect::<Result<_, AppError>>()?,
        };
        let cached = LOCAL_BACKEND
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let remote_reranker = match ranking {
            crate::views::config::Ranking::ContextualRerank => {
                super::remote_reranker::connect(&self.reranking, &asset_root)
            }
            crate::views::config::Ranking::Fusion => None,
        };
        // With a worker to score on, the CPU model is needed only if the
        // worker fails: do not spend a core and its memory loading it.
        let worker = matches!(remote_reranker, Some(Ok(_)));
        let mut backend = match cached {
            Some(cached) if cached.key == key => cached.backend,
            _ => Backend::load_local_search(&asset_root, ranking, worker)?,
        };
        backend.remote_reranker = remote_reranker;
        backend.assets_lease = selected.lease;
        Ok(Some(LoadedBackend {
            backend: Some(backend),
            cache_key: Some(key),
        }))
    }

    #[tracing::instrument(name = "load_embedding_models", skip_all)]
    pub fn load_indexer(&self) -> Result<Option<indexer::LoadedIndexer>, AppError> {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(pool) = super::test_embeddings::load(&self.root) {
            return Ok(Some(pool));
        }
        if !self.root.join(MARKER_NAME).is_file() {
            if let Some(cache) = indexer::current() {
                cache.clear();
            }
            return Ok(None);
        }
        let selected = generations::select(&self.root)?;
        let key = indexer::AssetsKey {
            accelerated: self.accelerated_embedding,
            root: selected.root.clone(),
            marker_modified: fs::metadata(self.root.join(MARKER_NAME))
                .map_err(AppError::io)?
                .modified()
                .ok(),
            files: selected
                .marker
                .files
                .into_iter()
                .map(|file| {
                    let modified = fs::metadata(selected.root.join(&file.path))
                        .map_err(AppError::io)?
                        .modified()
                        .ok();
                    Ok((file.path, file.size, modified))
                })
                .collect::<Result<_, AppError>>()?,
        };
        let cache = indexer::current();
        let mut pool = match cache.as_ref().and_then(|cache| cache.take(&key)) {
            Some(pool) => pool,
            None => EmbeddingPool::load_local(
                &selected.root,
                cache.is_some(),
                self.accelerated_embedding,
            )?,
        };
        pool.assets_lease = selected.lease;
        Ok(Some(indexer::LoadedIndexer::new(pool, key, cache)))
    }

    fn read_valid_marker(&self) -> Result<InstallMarker, AppError> {
        assets::read_valid_marker(&self.root)
    }
}

pub(super) fn load_reranker(root: &Path) -> Result<TextRerank, AppError> {
    load_reranker_with_threads(root, None)
}

fn load_reranker_with_threads(root: &Path, threads: Option<usize>) -> Result<TextRerank, AppError> {
    let reranker_snapshot = snapshot(root, RERANKER_REPOSITORY)?;
    let reranker_info = TextRerank::get_model_info(&RERANKER_MODEL);
    let reranker_model = UserDefinedRerankingModel::new(
        reranker_snapshot.join(reranker_info.model_file),
        read_tokenizer_files(&reranker_snapshot)?,
    );
    let reranker = TextRerank::try_new_from_user_defined(
        reranker_model,
        threads.map_or_else(RerankInitOptionsUserDefined::default, |threads| {
            RerankInitOptionsUserDefined::default().with_intra_threads(threads)
        }),
    )
    .map_err(|error| AppError::model(format!("failed to load reranking model: {error}")))?;
    Ok(reranker)
}

impl Backend {
    fn download_and_load(root: &Path) -> Result<Self, AppError> {
        downloads::local(root)?;
        Self::load_local(root)
    }

    pub(super) fn load_local(root: &Path) -> Result<Self, AppError> {
        Self::load_local_search(root, crate::views::config::Ranking::default(), false)
    }

    fn load_local_search(
        root: &Path,
        ranking: crate::views::config::Ranking,
        worker: bool,
    ) -> Result<Self, AppError> {
        // A command-line search loads both models, each single-threaded work
        // on its own files. Start the reranker now and wait for it only when
        // reranking begins: embedding the query and retrieval need the
        // embedding model alone.
        let reranker =
            matches!(ranking, crate::views::config::Ranking::ContextualRerank).then(|| {
                if worker {
                    reranker::Lazy::deferred(root)
                } else {
                    reranker::Lazy::start(root)
                }
            });
        let embedding = load_local_embedding(root, None)?;
        Ok(Self {
            embedding,
            reranker,
            remote_reranker: None,
            assets_lease: None,
        })
    }

    #[tracing::instrument(name = "embed", skip_all)]
    pub fn embed<S: AsRef<str> + Send + Sync>(
        &mut self,
        texts: &[S],
    ) -> Result<Vec<Vec<f32>>, AppError> {
        crate::observability::retrieval::measure_vec(
            crate::observability::retrieval::Stage::Embed,
            Some(texts.len()),
            || {
                self.embedding.embed(texts, None).map_err(|error| {
                    AppError::model(format!("embedding inference failed: {error}"))
                })
            },
        )
    }

    #[tracing::instrument(name = "rerank_inference", skip_all)]
    pub fn rerank<S: AsRef<str> + Send + Sync>(
        &mut self,
        query: S,
        documents: &[S],
    ) -> Result<Vec<fastembed::RerankResult>, AppError> {
        // Wait for the model outside the measured stage: loading is not
        // inference, and a search that started it late should not report a
        // slow reranker.
        self.wait_for_reranker()?;
        let result = crate::observability::retrieval::measure_vec(
            crate::observability::retrieval::Stage::Rerank,
            Some(documents.len()),
            || self.rerank_batched(query, documents, RERANK_BATCH),
        );
        if self.reranker.is_some() {
            crate::observability::retrieval::record_reranker(
                crate::observability::retrieval::RerankerEngine::Cpu,
                None,
            );
        }
        result
    }

    /// Blocks until the CPU reranker has loaded, if this backend has one.
    pub fn wait_for_reranker(&mut self) -> Result<(), AppError> {
        if let Some(reranker) = self.reranker.as_mut() {
            reranker.get()?;
        }
        Ok(())
    }

    pub fn remote_reranker(&self) -> RemoteReranker {
        self.remote_reranker.clone()
    }

    /// Raw CPU logits in input order, for one complete reranking pass.
    pub fn rerank_cpu(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f32>, AppError> {
        let mut scores = vec![f32::NEG_INFINITY; documents.len()];
        for result in self.rerank_batched(query, documents, RERANK_BATCH)? {
            *scores
                .get_mut(result.index)
                .ok_or_else(|| AppError::model("reranker returned an invalid document index"))? =
                result.score;
        }
        Ok(scores)
    }

    fn rerank_batched<S: AsRef<str> + Send + Sync>(
        &mut self,
        query: S,
        documents: &[S],
        batch_size: usize,
    ) -> Result<Vec<fastembed::RerankResult>, AppError> {
        if batch_size == 0 {
            return Err(AppError::internal("reranking batch size must be positive"));
        }
        let reranker = self
            .reranker
            .as_mut()
            .ok_or_else(|| AppError::internal("query backend has no reranker"))?
            .get()?;
        let mut order = (0..documents.len()).collect::<Vec<_>>();
        // Neighbours in length share a batch, so a long history message does
        // not pad every short candidate and little padding is computed.
        order.sort_by_key(|&index| documents[index].as_ref().len());
        let mut scores = Vec::with_capacity(documents.len());
        for indices in order.chunks(batch_size) {
            let refs = indices
                .iter()
                .map(|&index| documents[index].as_ref())
                .collect::<Vec<_>>();
            let results = reranker.run(query.as_ref(), &refs)?;
            for mut result in results {
                result.index = *indices.get(result.index).ok_or_else(|| {
                    AppError::model("reranker returned an invalid document index")
                })?;
                scores.push(result);
            }
        }
        scores.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.index.cmp(&b.index))
        });
        Ok(scores)
    }

    /// Focus long candidates on query evidence before the tokenizer truncates
    /// them. Keep the model's raw logits: they are not calibrated probabilities.
    #[cfg(test)]
    pub fn rerank_search(
        &mut self,
        query: &str,
        documents: &[&str],
    ) -> Result<Vec<fastembed::RerankResult>, AppError> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        let windows = documents
            .iter()
            .map(|document| crate::search_results::rerank_document(document, query))
            .collect::<Vec<_>>();
        let refs = windows.iter().map(AsRef::as_ref).collect::<Vec<_>>();
        self.rerank(query, &refs)
    }
}

pub(super) fn load_local_embedding(
    root: &Path,
    intra_threads: Option<usize>,
) -> Result<TextEmbedding, AppError> {
    let options = intra_threads.map_or_else(InitOptionsUserDefined::default, |threads| {
        InitOptionsUserDefined::default().with_intra_threads(threads)
    });
    load_local_embedding_with_options(root, options)
}

fn load_local_embedding_with_options(
    root: &Path,
    options: InitOptionsUserDefined,
) -> Result<TextEmbedding, AppError> {
    let embedding_snapshot = snapshot(root, EMBEDDING_REPOSITORY)?;
    let embedding_info = TextEmbedding::get_model_info(&EMBEDDING_MODEL)
        .map_err(|error| AppError::model(format!("unknown embedding model: {error}")))?;
    let mut embedding_model = UserDefinedEmbeddingModel::new(
        read_model_file(&embedding_snapshot.join(&embedding_info.model_file))?,
        read_tokenizer_files(&embedding_snapshot)?,
    )
    .with_quantization(TextEmbedding::get_quantization_mode(&EMBEDDING_MODEL));
    embedding_model.pooling = TextEmbedding::get_default_pooling_method(&EMBEDDING_MODEL);
    embedding_model
        .output_key
        .clone_from(&embedding_info.output_key);
    TextEmbedding::try_new_from_user_defined(embedding_model, options)
        .map_err(|error| AppError::model(format!("failed to load embedding model: {error}")))
}

pub(super) const fn coreml_embedding_enabled() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
}

const fn embedding_model_description() -> &'static str {
    if coreml_embedding_enabled() {
        "sentence-transformers/all-MiniLM-L6-v2 (Core ML FP32)"
    } else {
        "sentence-transformers/all-MiniLM-L6-v2 (quantized ONNX)"
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
