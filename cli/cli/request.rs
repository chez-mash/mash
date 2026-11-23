//! Turn parsed command-line arguments into the typed core request.
use super::{
    AppError, Cli, Command, IndexRequest, ModelsCommand, Operation, Paths, QdrantCommand, Request,
};
use crate::provider::Provider;

impl SearchArguments {
    pub(crate) fn into_selection(self) -> Result<crate::search_filter::Selection, AppError> {
        let filter = self
            .filter
            .map(|json| {
                if json.len() > 65536 {
                    return Err(AppError::usage("search filter exceeds 64 KiB"));
                }
                serde_json::from_str(&json)
                    .map_err(|_| AppError::usage("invalid structured search filter"))
            })
            .transpose()?;
        Ok(crate::search_filter::Selection {
            match_mode: self.match_mode,
            sort: self.sort,
            provider: self.provider,
            days: self.days,
            node: self.node,
            role: self.role,
            chat: self.chat,
            project: self.project,
            repository: self.repository,
            after: self
                .after
                .as_deref()
                .map(crate::search_filter::timestamp)
                .transpose()?,
            before: self
                .before
                .as_deref()
                .map(crate::search_filter::timestamp)
                .transpose()?,
            filter,
        })
    }
}

impl Cli {
    pub(super) fn paths(&self) -> Paths {
        Paths {
            config: self.config.clone(),
            db: self.db.clone(),
            models_dir: self.models_dir.clone(),
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "Keep the mechanical CLI-to-core command mapping together."
    )]
    pub(super) fn into_request(self) -> Result<Request, AppError> {
        let paths = self.paths();
        let operation = match self.command {
            Command::Init {
                force,
                no_agents,
                no_index,
                backfill_memory,
                marketplace,
                join,
                yes: _,
            } => {
                let join = join
                    .map(|text| crate::shared::join::Join::decode(&text))
                    .transpose()?;
                Operation::Init(crate::core::InitRequest {
                    force,
                    agents: !no_agents,
                    index: !no_index,
                    marketplace,
                    plan: join
                        .as_ref()
                        .map(|join| crate::core::join_plan(&paths, join))
                        .transpose()?,
                    join,
                    selected_agents: None,
                    backfill_memory,
                    agent_path: None,
                })
            }
            Command::ListChats {
                input,
                local,
                shared,
                diagnostics,
            } => Operation::ListChats {
                input,
                local,
                shared,
                diagnostics,
            },
            Command::Index {
                rebuild_local_views,
                full,
                provider,
                since_days,
                all_history,
                reread,
                backfill,
                ..
            } => Operation::Index(IndexRequest {
                mode: crate::core::IndexMode::from_rebuild_flag(rebuild_local_views),
                full,
                provider,
                since_days,
                all_history,
                reading: if reread {
                    crate::core::Reading::Everything
                } else {
                    crate::core::Reading::Changed
                },
                defer_publication: false,
                backfill,
            }),
            Command::Search {
                query,
                limit,
                filters,
                live: true,
                ..
            } => Operation::LiveSearch {
                query,
                limit,
                filters: Box::new(filters.into_selection()?),
            },
            Command::Search {
                shared_vector_search,
                shared_hnsw_ef,
                query,
                intent,
                limit,
                offset,
                unit,
                fusion_rank_constant,
                lexical_weight,
                semantic_weight,
                recency_weight,
                ranking,
                diagnostics,
                filters,
                shared,
                local,
                ..
            } => Operation::Search {
                shared_vector_search: shared_vector_search
                    .map(|algorithm| algorithm.request(shared_hnsw_ef))
                    .transpose()?,
                ranking,
                query,
                intent,
                limit,
                offset,
                unit,
                diagnostics,
                fusion: crate::views::config::FusionOverrides {
                    rank_constant: fusion_rank_constant,
                    lexical_weight,
                    semantic_weight,
                    recency_weight,
                },
                filters: Box::new(filters.into_selection()?),
                shared,
                local,
            },
            Command::View {
                id,
                context,
                shared,
            } => Operation::View {
                id,
                context,
                shared,
            },
            Command::Status {
                installation,
                agent,
                probe,
                shared,
                coverage,
                scope,
            } => {
                if installation {
                    Operation::Installation {
                        agents: agent,
                        probe: if probe {
                            Some(crate::mcp::guidance::fingerprint().map_err(|_| {
                                AppError::internal("cannot encode this CLI's MCP contract")
                            })?)
                        } else {
                            None
                        },
                    }
                } else {
                    Operation::Status {
                        shared,
                        coverage: coverage.then_some(*scope),
                    }
                }
            }
            Command::Doctor => Operation::Doctor,
            Command::Forget { id } => Operation::Forget { id },
            Command::Query {
                sql,
                request,
                limit,
                shared,
                node,
                timeout_seconds,
            } => {
                let parsed = |request: String| {
                    serde_json::from_str(&request)
                        .map_err(|error| AppError::usage(format!("invalid query request: {error}")))
                };
                if shared {
                    let Some(request) = request else {
                        return Err(AppError::usage("--shared requires --request"));
                    };
                    Operation::QueryShared {
                        request: parsed(request)?,
                        node,
                    }
                } else if let Some(request) = request {
                    Operation::QueryStructured {
                        request: parsed(request)?,
                        timeout_seconds,
                    }
                } else {
                    Operation::QuerySilver {
                        sql: sql.unwrap_or_default(),
                        limit: limit.unwrap_or(200),
                        timeout_seconds,
                    }
                }
            }
            Command::Export { directory } => Operation::ExportSilver { directory },
            Command::Import {
                directory,
                into,
                keep_identity,
            } => Operation::ImportSilver {
                directory,
                database: into,
                keep_identity,
            },
            Command::Models {
                command: ModelsCommand::Install,
            } => Operation::ModelsInstall,
            Command::Qdrant {
                command: QdrantCommand::Install,
            } => Operation::QdrantInstall,
            Command::Qdrant {
                command: QdrantCommand::Uninstall { delete_storage },
            } => Operation::QdrantUninstall { delete_storage },
            Command::Uninstall { yes, keep_history } => {
                Operation::Uninstall(crate::core::uninstall::Request {
                    apply: yes,
                    keep_history,
                })
            }
            Command::Mcp { .. } | Command::Dash { .. } => {
                return Err(AppError::usage(
                    "server commands must run as process commands",
                ));
            }
        };
        Ok(Request { paths, operation })
    }
}

/// The filters `mash search` takes, shared by full and live search.
#[derive(Debug, Clone, Default, clap::Args)]
pub(super) struct SearchArguments {
    /// Lexical matching: flexible (prefixes and typos), strict, prefix, or typo.
    #[arg(long, value_enum, default_value = "flexible")]
    match_mode: crate::views::matching::Mode,
    /// Order qualifying results by relevance (default) or recent (newest first).
    #[arg(long, value_enum, default_value = "relevance")]
    sort: crate::search_filter::Sort,
    #[arg(long)]
    provider: Vec<Provider>,
    #[arg(long)]
    days: Option<u32>,
    #[arg(long)]
    node: Vec<String>,
    #[arg(long)]
    role: Vec<crate::role::Role>,
    #[arg(long = "chat")]
    chat: Vec<String>,
    /// Exact recorded working directory; repeat for alternatives.
    #[arg(long)]
    project: Vec<String>,
    /// Exact Git origin URL; repeat for alternatives.
    #[arg(long)]
    repository: Vec<String>,
    /// Inclusive date or RFC3339 timestamp.
    #[arg(long)]
    after: Option<String>,
    /// Exclusive date or RFC3339 timestamp.
    #[arg(long)]
    before: Option<String>,
    /// JSON Boolean expression using all/any/not and metadata predicates.
    #[arg(long)]
    filter: Option<String>,
}
