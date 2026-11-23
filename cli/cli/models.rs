//! `mash models …`: install the semantic retrieval models.
use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub(super) enum ModelsCommand {
    /// Explicitly install embedding and reranking models.
    #[command(long_about = concat!(
        "Explicitly install embedding and reranking models.\n\n",
        "Downloads about 177 MB (about 90 MB more on Apple silicon and Windows, which add the embedding model their GPU engines run) ",
        "from https://huggingface.co, or from the Hugging Face mirror named by HF_ENDPOINT, ",
        "into the models directory, ~/.mash/models unless --models-dir is set. ",
        "Word-exact search does not need these models.",
    ))]
    Install,
}
