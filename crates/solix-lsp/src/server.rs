//! Main Solix Language Server implementation wiring all 32 LSP features.

#![forbid(unsafe_code)]

use crate::code_actions::code_actions;
use crate::code_lens::code_lenses;
use crate::completion::{completions, resolve_completion_item};
use crate::folding::folding_ranges;
use crate::formatter::{format_document, format_range};
use crate::hierarchy::{
    incoming_calls, outgoing_calls, prepare_call_hierarchy, prepare_type_hierarchy, subtypes,
    supertypes,
};
use crate::hover::hover;
use crate::inlay_hints::inlay_hints;
use crate::navigation::{
    document_highlights, document_links, find_definition, find_implementations, find_references,
    find_type_definition,
};
use crate::rename::{prepare_rename, rename_symbol_checked};
use crate::selection_range::selection_ranges;
use crate::semantic_tokens::{semantic_tokens_full, semantic_tokens_legend, semantic_tokens_range};
use crate::signature_help::signature_help;
use crate::symbols::{document_symbols, workspace_symbols};
use crate::workspace::WorkspaceManager;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::{
    GotoImplementationParams, GotoImplementationResponse, GotoTypeDefinitionParams,
    GotoTypeDefinitionResponse,
};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Official Solix Language Server backend.
#[derive(Debug)]
pub struct SolixLanguageServer {
    /// LSP client handle for publishing diagnostics, logs, and progress.
    client: Client,
    /// Workspace state and document overlay manager.
    workspace: Arc<WorkspaceManager>,
    /// Active debounced diagnostic tasks indexed by document URI.
    debounce_tasks: Arc<tokio::sync::Mutex<HashMap<Url, tokio::task::JoinHandle<()>>>>,
}

impl SolixLanguageServer {
    /// Creates a new language server instance with the given client and workspace manager.
    #[must_use]
    pub fn new(client: Client, workspace: Arc<WorkspaceManager>) -> Self {
        Self {
            client,
            workspace,
            debounce_tasks: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Accessor for the underlying workspace manager.
    #[must_use]
    pub fn workspace(&self) -> &Arc<WorkspaceManager> {
        &self.workspace
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SolixLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root_uri) = params.root_uri
            && let Ok(root_path) = root_uri.to_file_path()
        {
            self.workspace.set_workspace_root(root_path);
        }

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            definition_provider: Some(OneOf::Left(true)),
            type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
            implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
            references_provider: Some(OneOf::Left(true)),
            document_highlight_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            document_formatting_provider: Some(OneOf::Left(true)),
            document_range_formatting_provider: Some(OneOf::Left(true)),
            document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
                first_trigger_character: "}".to_string(),
                more_trigger_character: Some(vec![";".to_string(), "\n".to_string()]),
            }),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    ".".to_string(),
                    ":".to_string(),
                    "@".to_string(),
                    "<".to_string(),
                ]),
                all_commit_characters: None,
                work_done_progress_options: WorkDoneProgressOptions::default(),
                completion_item: None,
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string(), "<".to_string()]),
                retrigger_characters: None,
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            inlay_hint_provider: Some(OneOf::Left(true)),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    SemanticTokensRegistrationOptions {
                        text_document_registration_options: TextDocumentRegistrationOptions {
                            document_selector: Some(vec![DocumentFilter {
                                language: Some("solix".to_string()),
                                scheme: Some("file".to_string()),
                                pattern: None,
                            }]),
                        },
                        semantic_tokens_options: SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                            legend: semantic_tokens_legend(),
                            range: Some(true),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                        static_registration_options: StaticRegistrationOptions::default(),
                    },
                ),
            ),
            call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "solix-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Solix Language Server v{} ready", env!("CARGO_PKG_VERSION")),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let content = params.text_document.text;

        let workspace = self.workspace.clone();
        let target_uri = uri.clone();
        let snapshot_result =
            tokio::task::spawn_blocking(move || workspace.did_open(target_uri, version, content))
                .await;

        if let Ok(snapshot) = snapshot_result {
            self.client
                .publish_diagnostics(uri, snapshot.diagnostics, Some(version))
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().last() {
            // Update document buffer in memory immediately so completion/hover see newest text
            self.workspace
                .update_document(&uri, version, change.text.clone());

            // Cancel previous debounced analysis if still pending
            let mut tasks = self.debounce_tasks.lock().await;
            if let Some(previous_task) = tasks.remove(&uri) {
                previous_task.abort();
            }

            let client = self.client.clone();
            let workspace = self.workspace.clone();
            let target_uri = uri.clone();
            let content = change.text;

            let debounce_handle = tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                let analysis_uri = target_uri.clone();
                let snapshot_opt = tokio::task::spawn_blocking(move || {
                    workspace.analyze_document_change(&analysis_uri, version, &content)
                })
                .await
                .ok();

                if let Some(snapshot) = snapshot_opt {
                    client
                        .publish_diagnostics(target_uri, snapshot.diagnostics, Some(version))
                        .await;
                }
            });

            tasks.insert(uri, debounce_handle);
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        // Cancel pending debounce on save and run immediately
        let mut tasks = self.debounce_tasks.lock().await;
        if let Some(previous_task) = tasks.remove(&uri) {
            previous_task.abort();
        }
        drop(tasks);

        let workspace = self.workspace.clone();
        let target_uri = uri.clone();

        if let Some(text) = params.text {
            let doc_version = self
                .workspace
                .get_document(&uri)
                .map_or(1, |doc| doc.version);
            let snapshot_opt = tokio::task::spawn_blocking(move || {
                workspace.did_change(&target_uri, doc_version, text)
            })
            .await
            .ok();
            if let Some(snapshot) = snapshot_opt {
                self.client
                    .publish_diagnostics(uri, snapshot.diagnostics, Some(doc_version))
                    .await;
            }
        } else {
            let snapshot_opt =
                tokio::task::spawn_blocking(move || workspace.recompile_document(&target_uri))
                    .await
                    .ok()
                    .flatten();
            if let Some(snapshot) = snapshot_opt {
                self.client
                    .publish_diagnostics(uri, snapshot.diagnostics, Some(snapshot.version))
                    .await;
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut tasks = self.debounce_tasks.lock().await;
        if let Some(previous_task) = tasks.remove(&params.text_document.uri) {
            previous_task.abort();
        }
        self.workspace.did_close(&params.text_document.uri);
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for change in params.changes {
            if let Some(snapshot) = self.workspace.recompile_document(&change.uri) {
                self.client
                    .publish_diagnostics(change.uri, snapshot.diagnostics, Some(snapshot.version))
                    .await;
            }
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(find_definition(&self.workspace, &snapshot, position))
    }

    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(find_type_definition(&self.workspace, &snapshot, position))
    }

    async fn goto_implementation(
        &self,
        params: GotoImplementationParams,
    ) -> Result<Option<GotoImplementationResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        let locs = find_implementations(&self.workspace, &snapshot, position);
        Ok(locs.map(GotoImplementationResponse::Array))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_decl = params.context.include_declaration;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(find_references(
            &self.workspace,
            &snapshot,
            position,
            include_decl,
        ))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(document_highlights(&snapshot, position))
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(prepare_call_hierarchy(&snapshot, position))
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        Ok(incoming_calls(&self.workspace, &params.item))
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        Ok(outgoing_calls(&self.workspace, &params.item))
    }

    async fn prepare_type_hierarchy(
        &self,
        params: TypeHierarchyPrepareParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(prepare_type_hierarchy(&snapshot, position))
    }

    async fn supertypes(
        &self,
        params: TypeHierarchySupertypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        Ok(supertypes(&self.workspace, &params.item))
    }

    async fn subtypes(
        &self,
        params: TypeHierarchySubtypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        Ok(subtypes(&self.workspace, &params.item))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let snapshot = match self.workspace.get_live_snapshot(uri) {
            Some(snap) => snap,
            None => return Ok(None),
        };

        let list = completions(&self.workspace, &snapshot, &params);
        Ok(Some(CompletionResponse::List(list)))
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        if let Some(snapshot) = self.workspace.all_snapshots().into_iter().next() {
            Ok(resolve_completion_item(&self.workspace, &snapshot, params))
        } else {
            Ok(params)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_live_snapshot(uri) {
            Some(snap) => snap,
            None => return Ok(None),
        };

        Ok(hover(&snapshot, position))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let snapshot = match self.workspace.get_live_snapshot(uri) {
            Some(snap) => snap,
            None => return Ok(None),
        };

        Ok(signature_help(&snapshot, position))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = &params.text_document.uri;
        let range = params.range;
        let snapshot = match self.workspace.get_live_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(inlay_hints(&snapshot, range)))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;
        let snapshot = match self.workspace.get_live_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(semantic_tokens_full(&snapshot))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = &params.text_document.uri;
        let range = params.range;
        let snapshot = match self.workspace.get_live_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(semantic_tokens_range(&snapshot, range))
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = &params.text_document.uri;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(code_lenses(&self.workspace, &snapshot)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(document_symbols(&snapshot))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = &params.query;
        let response = workspace_symbols(&self.workspace, query);
        match response {
            Some(WorkspaceSymbolResponse::Nested(symbols)) => {
                let infos = symbols
                    .into_iter()
                    .map(|ws| {
                        let loc = match ws.location {
                            OneOf::Left(location) => location,
                            OneOf::Right(wsl) => Location {
                                uri: wsl.uri,
                                range: Range::default(),
                            },
                        };
                        #[allow(deprecated)]
                        SymbolInformation {
                            name: ws.name,
                            kind: ws.kind,
                            tags: ws.tags,
                            deprecated: None,
                            location: loc,
                            container_name: ws.container_name,
                        }
                    })
                    .collect();
                Ok(Some(infos))
            }
            _ => Ok(None),
        }
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = &params.text_document.uri;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(folding_ranges(&snapshot)))
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        let uri = &params.text_document.uri;
        let positions = &params.positions;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(selection_ranges(&snapshot, positions)))
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let uri = &params.text_document.uri;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(document_links(&self.workspace, &snapshot)))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(code_actions(&self.workspace, &snapshot, &params)))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = &params.text_document.uri;
        let position = params.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(prepare_rename(&snapshot, position))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        rename_symbol_checked(&self.workspace, &snapshot, position, new_name)
            .map(Some)
            .map_err(tower_lsp::jsonrpc::Error::invalid_params)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(format_document(&snapshot)))
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let range = params.range;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        Ok(Some(format_range(&snapshot, range)))
    }

    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document_position.text_document.uri;
        let cursor_position = params.text_document_position.position;
        let snapshot = match self.workspace.get_snapshot(uri) {
            Some(snapshot) => snapshot,
            None => return Ok(None),
        };

        let target_line = if params.ch == "\n" && cursor_position.line > 0 {
            cursor_position.line - 1
        } else {
            cursor_position.line
        };

        let line_range = Range {
            start: Position {
                line: target_line,
                character: 0,
            },
            end: Position {
                line: target_line + 1,
                character: 0,
            },
        };

        let localized_edits = format_range(&snapshot, line_range);
        Ok(Some(localized_edits))
    }
}
