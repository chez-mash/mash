//! DirectML embedding on a Windows GPU: the unquantized model, dynamic shapes.
use super::embeddings::EmbeddingGroup;
use super::models::{read_model_file, read_tokenizer_files, snapshot};
use super::{EMBEDDING_MODEL, EMBEDDING_REPOSITORY, FP32_MODEL_FILE};
use crate::AppError;
use fastembed::{
    InitOptionsUserDefined, QuantizationMode, TextEmbedding, UserDefinedEmbeddingModel,
};
use ort::ep;
use std::path::Path;

/// Load the model on the machine's GPU and run it once, so a machine without
/// a usable one is found out before indexing starts.
#[tracing::instrument(name = "load_directml_embedding", skip_all)]
pub(super) fn load(root: &Path) -> Result<TextEmbedding, AppError> {
    let embedding_snapshot = snapshot(root, EMBEDDING_REPOSITORY)?;
    let file = embedding_snapshot.join(FP32_MODEL_FILE);
    if !file.is_file() {
        return Err(AppError::model(
            "the GPU embedding model is not installed; run `mash models install` again",
        ));
    }
    let info = TextEmbedding::get_model_info(&EMBEDDING_MODEL)
        .map_err(|error| AppError::model(format!("unknown embedding model: {error}")))?;
    let mut model = UserDefinedEmbeddingModel::new(
        read_model_file(&file)?,
        read_tokenizer_files(&embedding_snapshot)?,
    )
    .with_quantization(QuantizationMode::None);
    model.pooling = TextEmbedding::get_default_pooling_method(&EMBEDDING_MODEL);
    model.output_key.clone_from(&info.output_key);
    // ONNX Runtime skips software adapters, so this fails where there is no
    // DirectX 12 GPU, and it keeps small shape operations on the CPU.
    let provider = ep::DirectML::default().build().error_on_failure();
    let options = InitOptionsUserDefined::default().with_execution_providers(vec![provider]);
    let mut embedding =
        TextEmbedding::try_new_from_user_defined(model, options).map_err(|error| {
            let detail = error.to_string();
            if detail.contains("No devices detected") {
                AppError::model("no DirectX 12 graphics card was found")
            } else {
                AppError::model(format!("DirectML could not load the model: {detail}"))
            }
        })?;
    embedding
        .embed(vec!["warm up"], None)
        .map_err(|error| AppError::model(format!("DirectML could not run the model: {error}")))?;
    Ok(embedding)
}

#[tracing::instrument(name = "directml_embedding_batch", skip_all, fields(inputs = groups.len()))]
pub(super) fn embed(
    model: &mut TextEmbedding,
    groups: &[EmbeddingGroup],
) -> Result<Vec<Vec<f32>>, AppError> {
    let texts = groups
        .iter()
        .map(|group| group.content.as_str())
        .collect::<Vec<_>>();
    // One wave is one device batch.
    model
        .embed(texts, Some(groups.len().max(1)))
        .map_err(|error| AppError::model(format!("DirectML embedding inference failed: {error}")))
}
