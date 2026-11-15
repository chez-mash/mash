//! Bounded, lazy fixed-shape Jina sessions; every graph node must use Core ML.
//! Only the reranking worker uses these: compiling and loading a shape takes
//! seconds, which a long-lived process pays once and a one-shot search cannot.
use super::cache;
use crate::AppError;
use crate::semantic::models::{read_tokenizer_files, snapshot};
use crate::semantic::{COREML_CACHE_DIRECTORY, RERANKER_MODEL, RERANKER_REPOSITORY};
use fastembed::{RerankInitOptionsUserDefined, TextRerank, UserDefinedRerankingModel};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const BATCH: usize = 4;
/// Compiled input lengths, in tokens. Cost grows with the padded length, and
/// in real searches most passages past 128 tokens stop short of 192, so that
/// shape saves them from being padded to 256. Each shape costs disk, memory
/// and start-up time, so there are no finer steps.
const SEQUENCES: [usize; 5] = [64, 128, 192, 256, 512];

fn bucket(tokens: usize) -> usize {
    SEQUENCES
        .into_iter()
        .find(|&length| length >= tokens)
        .unwrap_or(SEQUENCES[SEQUENCES.len() - 1])
}

struct Session {
    model: TextRerank,
    // Keep compiled artifacts leased until the SDK session has been dropped.
    _lease: fs::File,
}

pub(crate) struct Pool {
    root: PathBuf,
    tokenizer: tokenizers::Tokenizer,
    sessions: HashMap<usize, Session>,
}

impl Pool {
    pub(crate) fn new(root: &Path, mut tokenizer: tokenizers::Tokenizer) -> Self {
        tokenizer.with_padding(None);
        Self {
            root: root.to_path_buf(),
            tokenizer,
            sessions: HashMap::new(),
        }
    }

    /// Raw logits in input order for at most one batch of documents.
    pub(crate) fn run(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f32>, AppError> {
        if documents.is_empty() || documents.len() > BATCH {
            return Err(AppError::internal(
                "Core ML reranking takes one to four documents",
            ));
        }
        // Keep the CPU tokenizer's truncation policy, including pair special
        // tokens. Only padding changes; long query/document pairs stay equivalent.
        let pairs = documents
            .iter()
            .map(|document| (query, *document))
            .collect::<Vec<_>>();
        let encodings = self.tokenizer.encode_batch(pairs, true).map_err(|error| {
            AppError::model(format!("Core ML reranker tokenization failed: {error}"))
        })?;
        let sequence = bucket(
            encodings
                .iter()
                .map(tokenizers::Encoding::len)
                .max()
                .unwrap_or(0),
        );
        self.infer(sequence, query, documents)
    }

    /// Compile and load every shape, so no search waits for one.
    pub(crate) fn warm(&mut self) -> Result<(), AppError> {
        for sequence in SEQUENCES {
            self.infer(sequence, "warm up", &["warm up"])?;
        }
        Ok(())
    }

    fn infer(
        &mut self,
        sequence: usize,
        query: &str,
        documents: &[&str],
    ) -> Result<Vec<f32>, AppError> {
        let session = match self.sessions.entry(sequence) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(load(&self.root, sequence)?)
            }
        };
        // The compiled shape is exactly four pairs; the padding's scores are dropped.
        let mut padded = documents.to_vec();
        padded.resize(BATCH, "");
        let results = session
            .model
            .rerank(query, padded, false, Some(BATCH))
            .map_err(|error| {
                AppError::model(format!("Core ML reranking inference failed: {error}"))
            })?;
        if results.len() != BATCH {
            return Err(AppError::model(
                "Core ML reranker returned an unexpected result count",
            ));
        }
        let mut scores = vec![None; documents.len()];
        for result in results {
            if let Some(slot) = scores.get_mut(result.index) {
                *slot = Some(result.score);
            }
        }
        scores
            .into_iter()
            .collect::<Option<_>>()
            .ok_or_else(|| AppError::model("Core ML reranker returned too few scores"))
    }
}

fn load(root: &Path, sequence: usize) -> Result<Session, AppError> {
    let directory = root
        .join(COREML_CACHE_DIRECTORY)
        .join(format!("reranker-batch-{BATCH}-sequence-{sequence}"));
    let (model, lease) = cache::load(&directory, |directory| build(root, sequence, directory))?;
    Ok(Session {
        model,
        _lease: lease,
    })
}

fn build(root: &Path, sequence: usize, directory: &Path) -> Result<TextRerank, cache::BuildError> {
    let source = snapshot(root, RERANKER_REPOSITORY)?;
    let info = TextRerank::get_model_info(&RERANKER_MODEL);
    let model = UserDefinedRerankingModel::new(
        source.join(info.model_file),
        read_tokenizer_files(&source)?,
    );
    let provider = ort::ep::CoreML::default()
        .with_model_format(ort::ep::coreml::ModelFormat::MLProgram)
        .with_compute_units(ort::ep::coreml::ComputeUnits::All)
        .with_specialization_strategy(ort::ep::coreml::SpecializationStrategy::FastPrediction)
        .with_model_cache_dir(directory.to_string_lossy())
        .build()
        .error_on_failure();
    let sequence =
        i64::try_from(sequence).map_err(|_| AppError::internal("reranker shape exceeds i64"))?;
    let options = RerankInitOptionsUserDefined::default()
        .with_execution_providers(vec![provider])
        .with_disable_cpu_fallback(true)
        // Jina names these dimensions differently from MiniLM.
        .with_dimension_override("batch", 4)
        .with_dimension_override("sequence", sequence);
    let mut model = TextRerank::try_new_from_user_defined(model, options).map_err(|error| {
        let message = AppError::model(format!("failed to load Core ML reranker: {error}"));
        match error {
            fastembed::Error::Ort(_)
            | fastembed::Error::OrtBuilder(_)
            | fastembed::Error::OrtSession(_) => cache::BuildError::Compile(message),
            _ => cache::BuildError::Input(message),
        }
    })?;
    let mut padding = model.tokenizer.get_padding().cloned().ok_or_else(|| {
        AppError::model("Core ML reranker tokenizer has no padding configuration")
    })?;
    padding.strategy = tokenizers::PaddingStrategy::Fixed(
        usize::try_from(sequence).map_err(|_| AppError::internal("invalid reranker sequence"))?,
    );
    model.tokenizer.with_padding(Some(padding));
    Ok(model)
}
