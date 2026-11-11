//! In-memory document overlay store, workspace discovery, and multi-file compilation snapshot management.

#![forbid(unsafe_code)]

use crate::diagnostics::{
    binding_error_to_diagnostic, collect_dead_code_diagnostics, collect_unused_symbols_diagnostics,
    lexing_error_to_diagnostic, parsing_error_to_diagnostic,
};
use dashmap::DashMap;
use solix_common::Span;
use solix_common::solver::ModuleSolver;
use solix_common::tokens::Token;
use solix_common::types::Type;
use solix_compilation::CompilationContext::{BuildProfile, CompilationContext, CompilationOptions};
use solix_compilation::phases::binder::binder::bind_context;
use solix_compilation::phases::binder::layout::ClassLayout;
use solix_compilation::phases::binder::symbols::{SymbolEntry, SymbolId, SymbolTable};
use solix_compilation::phases::lexing::lexer::tokenize_source;
use solix_compilation::phases::parser::parse_context_resilient;
use solix_compilation::phases::parser::statements::declarations::ClassDeclaration;
use solix_compilation::phases::parser::statements::ProgramAst;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_lsp::lsp_types::{Diagnostic, Url};

/// In-memory editor buffer maintaining live unsaved document contents.
#[derive(Debug, Clone)]
pub struct VersionedDocument {
    /// Canonical URI identifier for the document.
    pub uri: Url,
    /// Monotonically increasing document version number reported by client.
    pub version: i32,
    /// Current text content in memory.
    pub content: String,
}

/// Frozen snapshot of semantic analysis artifacts from a successful or partial binder execution.
#[derive(Debug, Clone)]
pub struct BoundSemanticSnapshot {
    /// Symbol table containing all discovered and declared symbols.
    pub symbol_table: SymbolTable,
    /// Calculated memory layouts and VTable maps for all classes.
    pub layouts: HashMap<SymbolId, ClassLayout>,
    /// Evaluated semantic types of AST expressions indexed by coordinate span.
    pub node_types: HashMap<Span, Type>,
    /// Usage spans resolved to declared symbol identifiers.
    pub resolved_symbols: HashMap<Span, SymbolId>,
    /// Synthesized ASTs of generic classes monomorphized at compile-time.
    pub monomorphized_classes: Vec<ClassDeclaration>,
}

/// Compilation snapshot for a single source file, preserving last-known-good semantics during incomplete typing.
#[derive(Debug, Clone)]
pub struct AnalysisSnapshot {
    /// Canonical URI of this source unit.
    pub uri: Url,
    /// Editor document version at the time of analysis.
    pub version: i32,
    /// Assigned numeric source file identifier in the compilation context.
    pub source_id: u16,
    /// Complete text of the analyzed source file.
    pub source_text: String,
    /// Token stream produced by the lexical analyzer.
    pub tokens: Vec<Token>,
    /// Diagnostics accumulated across lexer, parser, and semantic binder passes.
    pub diagnostics: Vec<Diagnostic>,
    /// Resilient AST produced by the parser.
    pub ast: Option<ProgramAst>,
    /// Last successfully bound or partially bound semantic snapshot.
    pub last_good_semantic: Option<Arc<BoundSemanticSnapshot>>,
}

impl AnalysisSnapshot {
    /// Searches for a declared or referenced symbol at the given byte offset.
    #[must_use]
    pub fn symbol_at_offset(&self, offset: usize) -> Option<SymbolEntry> {
        let semantic = self.last_good_semantic.as_ref()?;
        let target_offset = offset as u32;

        // 1. Check resolved symbol usages at this offset (smallest matching span)
        let mut best_usage_match: Option<(Span, SymbolId)> = None;
        for (usage_span, symbol_id) in &semantic.resolved_symbols {
            if usage_span.start <= target_offset && target_offset <= usage_span.end {
                if let Some((current_span, _)) = best_usage_match {
                    if usage_span.length() < current_span.length() {
                        best_usage_match = Some((*usage_span, *symbol_id));
                    }
                } else {
                    best_usage_match = Some((*usage_span, *symbol_id));
                }
            }
        }

        if let Some((_, resolved_id)) = best_usage_match
            && let Some(symbol_entry) = semantic.symbol_table.get_symbol(resolved_id) {
                return Some(symbol_entry.clone());
            }

        // 2. Check declared symbols whose name or span contains the offset (smallest matching span)
        let mut best_declared_match: Option<SymbolEntry> = None;
        for symbol_entry in &semantic.symbol_table.symbols {
            if symbol_entry.source_id == self.source_id
                && symbol_entry.span.start <= target_offset
                && target_offset <= symbol_entry.span.end
            {
                if let Some(ref current) = best_declared_match {
                    if symbol_entry.span.length() < current.span.length() {
                        best_declared_match = Some(symbol_entry.clone());
                    }
                } else {
                    best_declared_match = Some(symbol_entry.clone());
                }
            }
        }

        best_declared_match
    }

    /// Evaluates the semantic type of the expression or symbol at the given byte offset.
    #[must_use]
    pub fn type_at_offset(&self, offset: usize) -> Option<Type> {
        let semantic = self.last_good_semantic.as_ref()?;
        let target_offset = offset as u32;

        let mut best_match: Option<(Span, Type)> = None;
        for (node_span, evaluated_type) in &semantic.node_types {
            if node_span.start <= target_offset && target_offset <= node_span.end {
                if let Some((current_best_span, _)) = &best_match {
                    if node_span.length() < current_best_span.length() {
                        best_match = Some((*node_span, evaluated_type.clone()));
                    }
                } else {
                    best_match = Some((*node_span, evaluated_type.clone()));
                }
            }
        }

        if let Some((_, evaluated_type)) = best_match {
            return Some(evaluated_type);
        }

        if let Some(symbol) = self.symbol_at_offset(offset) {
            return Some(symbol.symbol_type);
        }

        None
    }

    /// Extracts doc comments (`///` or `/** ... */`) immediately preceding the given declaration span.
    #[must_use]
    pub fn extract_doc_comment_for_span(&self, span: Span) -> Option<String> {
        let start_offset = span.start as usize;
        if start_offset > self.source_text.len() {
            return None;
        }

        let prefix = &self.source_text[..start_offset];
        let lines: Vec<&str> = prefix.lines().collect();
        if lines.is_empty() {
            return None;
        }

        let mut doc_lines = Vec::new();
        let mut index = lines.len() - 1;

        // Skip the line of the declaration itself if span starts on it
        while index > 0 && lines[index].trim().is_empty() {
            index -= 1;
        }

        while index < lines.len() {
            let line_trimmed = lines[index].trim();
            if let Some(stripped) = line_trimmed.strip_prefix("///") {
                doc_lines.push(stripped.trim().to_string());
                if index == 0 {
                    break;
                }
                index -= 1;
            } else if line_trimmed.ends_with("*/") {
                // Multi-line block comment
                let mut block_lines = Vec::new();
                let mut block_index = index;
                let mut found_start = false;
                loop {
                    let raw_line = lines[block_index].trim();
                    let cleaned = raw_line
                        .trim_start_matches("/*")
                        .trim_end_matches("*/")
                        .trim_start_matches('*')
                        .trim();
                    if !cleaned.is_empty() {
                        block_lines.push(cleaned.to_string());
                    }
                    if raw_line.starts_with("/*") {
                        found_start = true;
                        break;
                    }
                    if block_index == 0 {
                        break;
                    }
                    block_index -= 1;
                }
                if found_start {
                    block_lines.reverse();
                    doc_lines.extend(block_lines);
                }
                break;
            } else {
                break;
            }
        }

        if doc_lines.is_empty() {
            None
        } else {
            doc_lines.reverse();
            Some(doc_lines.join("\n"))
        }
    }
}

/// Workspace state manager coordinating document overlays, manifest resolution, and multi-file analysis.
#[derive(Debug)]
pub struct WorkspaceManager {
    /// In-memory document buffers.
    documents: DashMap<Url, VersionedDocument>,
    /// Latest analysis snapshots indexed by document URI.
    snapshots: DashMap<Url, AnalysisSnapshot>,
    /// Optional root directory of the workspace.
    workspace_root: std::sync::RwLock<Option<PathBuf>>,
    /// Pluggable module solver for package dependencies and standard library resolution.
    solver: Option<Arc<dyn ModuleSolver>>,
}

impl WorkspaceManager {
    /// Initializes a new workspace manager with optional workspace root and module solver.
    #[must_use]
    pub fn new(workspace_root: Option<PathBuf>, solver: Option<Arc<dyn ModuleSolver>>) -> Self {
        Self {
            documents: DashMap::new(),
            snapshots: DashMap::new(),
            workspace_root: std::sync::RwLock::new(workspace_root),
            solver,
        }
    }

    /// Sets or updates the active workspace root directory.
    pub fn set_workspace_root(&self, root_path: PathBuf) {
        if let Ok(mut lock) = self.workspace_root.write() {
            *lock = Some(root_path);
        }
    }

    /// Returns the current workspace root directory, if configured.
    #[must_use]
    pub fn workspace_root(&self) -> Option<PathBuf> {
        self.workspace_root.read().ok().and_then(|guard| guard.clone())
    }

    /// Records an opened document buffer and performs an analysis pass.
    pub fn did_open(&self, uri: Url, version: i32, content: String) -> AnalysisSnapshot {
        self.documents.insert(
            uri.clone(),
            VersionedDocument {
                uri: uri.clone(),
                version,
                content: content.clone(),
            },
        );
        self.analyze_document(&uri, version, &content)
    }

    /// Updates an in-memory document buffer and performs an analysis pass.
    pub fn did_change(&self, uri: &Url, version: i32, content: String) -> AnalysisSnapshot {
        self.documents.insert(
            uri.clone(),
            VersionedDocument {
                uri: uri.clone(),
                version,
                content: content.clone(),
            },
        );
        self.analyze_document(uri, version, &content)
    }

    /// Cleans up document state when a document is closed.
    pub fn did_close(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Retrieves an in-memory document buffer if currently open.
    #[must_use]
    pub fn get_document(&self, uri: &Url) -> Option<VersionedDocument> {
        self.documents.get(uri).map(|entry| entry.value().clone())
    }

    /// Retrieves the latest analysis snapshot for the given document URI.
    #[must_use]
    pub fn get_snapshot(&self, uri: &Url) -> Option<AnalysisSnapshot> {
        self.snapshots.get(uri).map(|entry| entry.value().clone())
    }

    /// Returns all currently cached analysis snapshots across the workspace.
    #[must_use]
    pub fn all_snapshots(&self) -> Vec<AnalysisSnapshot> {
        self.snapshots
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Re-analyzes a document given its URI.
    pub fn recompile_document(&self, uri: &Url) -> Option<AnalysisSnapshot> {
        let document = self.get_document(uri)?;
        Some(self.analyze_document(uri, document.version, &document.content))
    }

    /// Searches upward from the target file path for an enclosing `manifest.toml`.
    #[must_use]
    pub fn find_project_manifest(file_path: &Path) -> Option<PathBuf> {
        let mut current_dir = if file_path.is_file() {
            file_path.parent()?
        } else {
            file_path
        };

        loop {
            let candidate = current_dir.join("manifest.toml");
            if candidate.is_file() {
                return Some(candidate);
            }
            current_dir = current_dir.parent()?;
        }
    }

    /// Discovers all `.slx` source files within `src/` and `tests/` of a project root.
    #[must_use]
    pub fn discover_project_sources(project_root: &Path) -> Vec<PathBuf> {
        let mut sources = Vec::new();
        let src_dir = project_root.join("src");
        if src_dir.is_dir() {
            collect_slx_files_recursive(&src_dir, &mut sources);
        }
        let tests_dir = project_root.join("tests");
        if tests_dir.is_dir() {
            collect_slx_files_recursive(&tests_dir, &mut sources);
        }
        sources
    }

    /// Compiles and analyzes a document, supporting both single-file and multi-file project modes.
    fn analyze_document(&self, target_uri: &Url, version: i32, current_content: &str) -> AnalysisSnapshot {
        let target_path = target_uri.to_file_path().ok();
        let manifest_path = target_path
            .as_ref()
            .and_then(|path| Self::find_project_manifest(path))
            .or_else(|| {
                self.workspace_root()
                    .and_then(|root| Self::find_project_manifest(&root))
            });

        let (sources_map, target_source_name) = if let Some(manifest) = manifest_path {
            let project_root = manifest.parent().unwrap_or_else(|| Path::new("."));
            let discovered_files = Self::discover_project_sources(project_root);
            let mut map = HashMap::new();

            for file_path in discovered_files {
                let file_url = Url::from_file_path(&file_path).ok();
                let file_name = file_path.to_string_lossy().to_string();

                let file_content = if let Some(url) = &file_url {
                    if url == target_uri {
                        current_content.to_string()
                    } else if let Some(open_doc) = self.documents.get(url) {
                        open_doc.content.clone()
                    } else {
                        std::fs::read_to_string(&file_path).unwrap_or_default()
                    }
                } else {
                    std::fs::read_to_string(&file_path).unwrap_or_default()
                };

                map.insert(file_name, file_content);
            }

            let target_name = target_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| target_uri.to_string());

            if !map.contains_key(&target_name) {
                map.insert(target_name.clone(), current_content.to_string());
            }

            (map, target_name)
        } else {
            let mut map = HashMap::new();
            let target_name = target_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| target_uri.to_string());
            map.insert(target_name.clone(), current_content.to_string());
            (map, target_name)
        };

        // Construct CompilationContext
        let mut options = CompilationOptions {
            profile: BuildProfile::Debug,
            include_builtins: true,
            codes: sources_map,
            ..Default::default()
        };
        if let Some(solver) = &self.solver {
            options = options.with_solver(solver.clone());
        }

        let mut context = CompilationContext::new(options);

        // Stage 1: Tokenize all sources
        let mut accumulated_diagnostics = Vec::new();
        let sources_count = context.sources.len();
        context.tokens.resize_with(sources_count, Vec::new);

        let mut target_source_id = 0u16;
        for (source_index, source_text) in context.sources.iter().enumerate() {
            let source_id = source_index as u16;
            let source_name = &context.source_names[source_index];
            if source_name == &target_source_name {
                target_source_id = source_id;
            }

            let (tokens, lex_errors) = tokenize_source(source_id, source_text);
            if source_id == target_source_id {
                for error in &lex_errors {
                    accumulated_diagnostics.push(lexing_error_to_diagnostic(source_text, error));
                }
            }
            context.tokens[source_index] = tokens;
        }

        // Stage 2: Resilient parsing
        let parse_errors = parse_context_resilient(&mut context);
        let target_source_text = context.sources[target_source_id as usize].clone();
        for error in &parse_errors {
            let error_span = error.source_span();
            if error_span.offset() < target_source_text.len() {
                accumulated_diagnostics.push(parsing_error_to_diagnostic(&target_source_text, error));
            }
        }

        // Stage 3: Semantic binding
        let bind_result = bind_context(&mut context);
        if let Err(bind_errors) = bind_result {
            for error in &bind_errors {
                accumulated_diagnostics.push(binding_error_to_diagnostic(&target_source_text, error));
            }
        }

        // Extract semantic artifacts
        let new_semantic_snapshot = context.binding_artifacts.map(|artifacts| {
            Arc::new(BoundSemanticSnapshot {
                symbol_table: artifacts.symbol_table,
                layouts: artifacts.layouts,
                node_types: artifacts.node_types,
                resolved_symbols: artifacts.resolved_symbols,
                monomorphized_classes: artifacts.monomorphized_classes,
            })
        });

        // Retain last good semantic snapshot if the current bind failed to produce one
        let last_good_semantic = new_semantic_snapshot.or_else(|| {
            self.snapshots
                .get(target_uri)
                .and_then(|snap| snap.last_good_semantic.clone())
        });

        // Stage 4: Dead code & unused symbol diagnostics
        let target_ast = context.ast.get(target_source_id as usize).and_then(|opt| opt.clone());
        if let Some(ast) = &target_ast {
            let dead_code_diags = collect_dead_code_diagnostics(&target_source_text, ast);
            accumulated_diagnostics.extend(dead_code_diags);
        }

        if let Some(semantic) = &last_good_semantic {
            let unused_diags = collect_unused_symbols_diagnostics(
                &target_source_text,
                target_source_id,
                &semantic.symbol_table,
                &semantic.resolved_symbols,
            );
            accumulated_diagnostics.extend(unused_diags);
        }

        let target_tokens = context
            .tokens
            .get(target_source_id as usize)
            .cloned()
            .unwrap_or_default();

        let final_snapshot = AnalysisSnapshot {
            uri: target_uri.clone(),
            version,
            source_id: target_source_id,
            source_text: target_source_text.clone(),
            tokens: target_tokens,
            diagnostics: accumulated_diagnostics,
            ast: target_ast,
            last_good_semantic,
        };

        self.snapshots.insert(target_uri.clone(), final_snapshot.clone());
        final_snapshot
    }
}

fn collect_slx_files_recursive(dir: &Path, result: &mut Vec<PathBuf>) {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_slx_files_recursive(&path, result);
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "slx") {
            result.push(path);
        }
    }
}
