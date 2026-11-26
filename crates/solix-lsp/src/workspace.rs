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
use solix_compilation::phases::parser::statements::ProgramAst;
use solix_compilation::phases::parser::statements::declarations::ClassDeclaration;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_lsp::lsp_types::{CompletionItemKind, Diagnostic, Url};

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
            && let Some(symbol_entry) = semantic.symbol_table.get_symbol(resolved_id)
        {
            return Some(symbol_entry.clone());
        }

        // 2. Check local variable and parameter declarations
        for local_entry in &semantic.symbol_table.local_symbols {
            if local_entry.source_id == self.source_id
                && local_entry.decl_span.start <= target_offset
                && target_offset <= local_entry.decl_span.end
                && let Some(entry) = semantic.symbol_table.get_symbol(local_entry.symbol_id)
            {
                return Some(entry.clone());
            }
        }

        // 3. Check declared symbols whose name or span contains the offset (smallest matching span)
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

        // Skip the line of the declaration itself if it does not contain a doc comment
        if index > 0
            && !lines[index].trim().starts_with("///")
            && !lines[index].trim().ends_with("*/")
        {
            index -= 1;
        }

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

/// Namespace and symbol index for import autocompletion within a package.
#[derive(Debug, Clone, Default)]
pub struct PackageNamespaceIndex {
    /// All full package paths declared in this module (e.g. `"solix.io"`, `"solix.math"`).
    pub namespaces: BTreeSet<String>,
    /// Map from dot-separated namespace path (e.g. `"solix.io"`) to exported public symbols
    /// `(symbol_name, CompletionItemKind, detail_signature)`.
    pub symbols_by_namespace: HashMap<String, Vec<(String, CompletionItemKind, String)>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageExportCacheKey {
    pub package_name: String,
    pub profile: String,
    pub version: String,
    pub archive_mtime_secs: u64,
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
    /// Cached package namespace export indexes for fast autocompletion.
    package_export_index:
        std::sync::RwLock<HashMap<String, (PackageExportCacheKey, PackageNamespaceIndex)>>,
    /// In-memory cache of resolved dependency programs to avoid disk re-scans.
    cached_dependencies: DashMap<String, solix_common::bytecode::BinaryProgram>,
}

impl WorkspaceManager {
    /// Initializes a new workspace manager with optional workspace root and module solver.
    #[must_use]
    pub fn new(workspace_root: Option<PathBuf>, solver: Option<Arc<dyn ModuleSolver>>) -> Self {
        let effective_solver: Option<Arc<dyn ModuleSolver>> = solver.or_else(|| {
            let store_directory = solix_common::solver::resolve_package_store_dir(None);
            Some(
                Arc::new(solix_common::solver::ClientModuleSolver::with_store_dir(
                    store_directory,
                )) as Arc<dyn ModuleSolver>,
            )
        });
        Self {
            documents: DashMap::new(),
            snapshots: DashMap::new(),
            workspace_root: std::sync::RwLock::new(workspace_root),
            solver: effective_solver,
            package_export_index: std::sync::RwLock::new(HashMap::new()),
            cached_dependencies: DashMap::new(),
        }
    }

    /// Returns the pluggable module solver if configured.
    #[must_use]
    pub fn solver(&self) -> Option<Arc<dyn ModuleSolver>> {
        self.solver.clone()
    }

    /// Attempts to find and parse the `ProjectManifest` corresponding to a document URI.
    #[must_use]
    pub fn project_manifest_for_uri(
        &self,
        target_uri: &Url,
    ) -> Option<solix_common::project::ProjectManifest> {
        let manifest_path = target_uri
            .to_file_path()
            .ok()
            .and_then(|file_path| Self::find_project_manifest(&file_path))
            .or_else(|| {
                self.workspace_root()
                    .and_then(|root_path| Self::find_project_manifest(&root_path))
            })?;
        solix_common::project::ProjectManifest::from_file(&manifest_path).ok()
    }

    /// Resolves or returns the cached package namespace and symbol index for the given package name.
    #[must_use]
    pub fn resolve_package_index(&self, target_package_name: &str) -> PackageNamespaceIndex {
        let mut cache_key = PackageExportCacheKey {
            package_name: target_package_name.to_string(),
            profile: "release".to_string(),
            version: "0.1.0".to_string(),
            archive_mtime_secs: 0,
        };

        if let Some(solver) = &self.solver
            && let Ok(solved_package) = solver.resolve_latest(target_package_name)
        {
            let store_directory = solix_common::solver::resolve_package_store_dir(None);
            let profile_name = if !solved_package.program.build_profile.is_empty() {
                &solved_package.program.build_profile
            } else {
                "release"
            };
            let archive_file = store_directory
                .join(format!(
                    "{target_package_name}-{profile_name}@{}",
                    solved_package.version
                ))
                .join(format!("{target_package_name}.slxarc"));
            let mtime_secs = std::fs::metadata(&archive_file)
                .and_then(|metadata| metadata.modified())
                .and_then(|time| {
                    time.duration_since(std::time::UNIX_EPOCH)
                        .map_err(std::io::Error::other)
                })
                .map_or(0, |duration| duration.as_secs());

            cache_key = PackageExportCacheKey {
                package_name: target_package_name.to_string(),
                profile: profile_name.to_string(),
                version: solved_package.version.to_string(),
                archive_mtime_secs: mtime_secs,
            };
        }

        // Check read cache with key validation
        if let Ok(guard) = self.package_export_index.read()
            && let Some((cached_key, cached_index)) = guard.get(target_package_name)
            && cached_key == &cache_key
        {
            return cached_index.clone();
        }

        let mut built_index = PackageNamespaceIndex::default();

        // 1. Ingest from ModuleSolver if available
        if let Some(solver) = &self.solver
            && let Ok(solved_package) = solver.resolve_latest(target_package_name)
        {
            let manifest = &solved_package.program.export_manifest;

            // Pre-parse API header declarations for fallback namespace mapping
            let mut api_header_namespaces: HashMap<String, String> = HashMap::new();
            if !manifest.api_header.is_empty() {
                let mut current_package = target_package_name.to_string();
                for raw_line in manifest.api_header.lines() {
                    let trimmed_line = raw_line.trim();
                    if let Some(pkg) = trimmed_line.strip_prefix("package ") {
                        let cleaned = pkg.trim_end_matches(';').trim().replace("::", ".");
                        current_package = cleaned;
                    } else if trimmed_line.starts_with("public class ")
                        || trimmed_line.starts_with("public interface ")
                        || trimmed_line.starts_with("public enum ")
                        || trimmed_line.starts_with("class ")
                        || trimmed_line.starts_with("interface ")
                        || trimmed_line.starts_with("enum ")
                    {
                        let words: Vec<&str> = trimmed_line.split_whitespace().collect();
                        if let Some(name_token) = words.get(2).or_else(|| words.get(1)) {
                            let clean_token = name_token.split('<').next().unwrap_or(name_token);
                            let clean_token = clean_token.trim_end_matches('{').trim();
                            if let Some(last_colons) = clean_token.rfind("::") {
                                let namespace_part = clean_token[..last_colons].replace("::", ".");
                                let simple_name = &clean_token[last_colons + 2..];
                                api_header_namespaces
                                    .insert(simple_name.to_string(), namespace_part);
                            } else if !clean_token.is_empty() {
                                api_header_namespaces
                                    .insert(clean_token.to_string(), current_package.clone());
                            }
                        }
                    }
                }
            }

            for exported_class in &manifest.exported_classes {
                if !exported_class
                    .modifiers
                    .contains(&solix_common::bytecode::manifest::DeclarationModifier::Public)
                {
                    continue;
                }
                let (namespace_path, class_name) = if exported_class.symbol.contains("::") {
                    split_symbol_namespace(&exported_class.symbol, target_package_name)
                } else if let Some(header_ns) = api_header_namespaces.get(&exported_class.name) {
                    (header_ns.clone(), exported_class.name.clone())
                } else {
                    split_symbol_namespace(&exported_class.symbol, target_package_name)
                };
                insert_namespace_and_ancestors(&mut built_index.namespaces, &namespace_path);
                let item_kind = if exported_class.is_interface {
                    CompletionItemKind::INTERFACE
                } else {
                    CompletionItemKind::CLASS
                };
                let detail_string = if exported_class.is_interface {
                    format!("interface {}", class_name)
                } else {
                    format!("class {}", class_name)
                };
                built_index
                    .symbols_by_namespace
                    .entry(namespace_path)
                    .or_default()
                    .push((class_name, item_kind, detail_string));
            }

            for exported_function in &manifest.exported_functions {
                if !exported_function
                    .modifiers
                    .contains(&solix_common::bytecode::manifest::DeclarationModifier::Public)
                {
                    continue;
                }
                if exported_function.is_constructor
                    || exported_function.is_destructor
                    || exported_function.is_operator
                {
                    continue;
                }
                let cleaned_symbol = exported_function
                    .symbol
                    .split('/')
                    .next()
                    .unwrap_or(&exported_function.symbol);
                let (namespace_path, func_name) = if cleaned_symbol.contains("::") {
                    split_symbol_namespace(cleaned_symbol, target_package_name)
                } else if let Some(header_ns) =
                    api_header_namespaces.get(&exported_function.simple_name)
                {
                    (header_ns.clone(), exported_function.simple_name.clone())
                } else {
                    split_symbol_namespace(cleaned_symbol, target_package_name)
                };
                insert_namespace_and_ancestors(&mut built_index.namespaces, &namespace_path);
                let detail_string = format!("fn {}(...)", func_name);
                built_index
                    .symbols_by_namespace
                    .entry(namespace_path)
                    .or_default()
                    .push((func_name, CompletionItemKind::FUNCTION, detail_string));
            }

            for exported_enum in &manifest.exported_enums {
                if !exported_enum
                    .modifiers
                    .contains(&solix_common::bytecode::manifest::DeclarationModifier::Public)
                {
                    continue;
                }
                let (namespace_path, enum_name) = if exported_enum.name.contains("::") {
                    split_symbol_namespace(&exported_enum.name, target_package_name)
                } else if let Some(header_ns) = api_header_namespaces.get(&exported_enum.name) {
                    (header_ns.clone(), exported_enum.name.clone())
                } else {
                    split_symbol_namespace(&exported_enum.name, target_package_name)
                };
                insert_namespace_and_ancestors(&mut built_index.namespaces, &namespace_path);
                let detail_string = format!("enum {}", enum_name);
                built_index
                    .symbols_by_namespace
                    .entry(namespace_path)
                    .or_default()
                    .push((enum_name, CompletionItemKind::ENUM, detail_string));
            }

            for exported_static in &manifest.exported_statics {
                if !exported_static
                    .modifiers
                    .contains(&solix_common::bytecode::manifest::DeclarationModifier::Public)
                    && !exported_static
                        .modifiers
                        .contains(&solix_common::bytecode::manifest::DeclarationModifier::Const)
                {
                    continue;
                }
                let (namespace_path, static_name) = if exported_static.symbol.contains("::") {
                    split_symbol_namespace(&exported_static.symbol, target_package_name)
                } else if let Some(header_ns) = api_header_namespaces.get(&exported_static.name) {
                    (header_ns.clone(), exported_static.name.clone())
                } else {
                    split_symbol_namespace(&exported_static.symbol, target_package_name)
                };
                insert_namespace_and_ancestors(&mut built_index.namespaces, &namespace_path);
                let detail_string = format!("static {}", static_name);
                built_index
                    .symbols_by_namespace
                    .entry(namespace_path)
                    .or_default()
                    .push((static_name, CompletionItemKind::VARIABLE, detail_string));
            }

            for template_item in &manifest.generic_templates {
                let (namespace_path, template_name) = if template_item.name.contains("::") {
                    split_symbol_namespace(&template_item.name, target_package_name)
                } else if let Some(header_ns) = api_header_namespaces.get(&template_item.name) {
                    (header_ns.clone(), template_item.name.clone())
                } else {
                    split_symbol_namespace(&template_item.name, target_package_name)
                };
                insert_namespace_and_ancestors(&mut built_index.namespaces, &namespace_path);
                let detail_string = format!("template class {}", template_name);
                built_index
                    .symbols_by_namespace
                    .entry(namespace_path)
                    .or_default()
                    .push((template_name, CompletionItemKind::CLASS, detail_string));
            }
        }

        // 2. Ingest from active workspace snapshots
        for snapshot in self.all_snapshots() {
            if let Some(ast) = &snapshot.ast {
                let package_path = if let Some(package_statement) = &ast.package {
                    let mut segments = Vec::new();
                    for span in &package_statement.path {
                        let start_index = span.start as usize;
                        let end_index = span.end as usize;
                        if end_index <= snapshot.source_text.len() {
                            segments.push(&snapshot.source_text[start_index..end_index]);
                        }
                    }
                    segments.join(".")
                } else {
                    String::new()
                };

                let matches_package = package_path == target_package_name
                    || package_path.starts_with(&format!("{target_package_name}."));

                if matches_package && !package_path.is_empty() {
                    insert_namespace_and_ancestors(&mut built_index.namespaces, &package_path);

                    for declaration in &ast.declarations {
                        match declaration {
                            solix_compilation::phases::parser::statements::declarations::Declaration::Class(class_declaration) => {
                                if class_declaration.modifiers.contains(&solix_compilation::phases::parser::statements::declarations::Modifier::Public) {
                                    let class_name = get_span_text_workspace(&snapshot.source_text, class_declaration.name_span);
                                    let detail_string = format!("class {}", class_name);
                                    built_index
                                        .symbols_by_namespace
                                        .entry(package_path.clone())
                                        .or_default()
                                        .push((
                                            class_name.to_string(),
                                            CompletionItemKind::CLASS,
                                            detail_string,
                                        ));
                                }
                            }
                            solix_compilation::phases::parser::statements::declarations::Declaration::Interface(interface_declaration) => {
                                if interface_declaration.modifiers.contains(&solix_compilation::phases::parser::statements::declarations::Modifier::Public) {
                                    let interface_name = get_span_text_workspace(&snapshot.source_text, interface_declaration.name_span);
                                    let detail_string = format!("interface {}", interface_name);
                                    built_index
                                        .symbols_by_namespace
                                        .entry(package_path.clone())
                                        .or_default()
                                        .push((
                                            interface_name.to_string(),
                                            CompletionItemKind::INTERFACE,
                                            detail_string,
                                        ));
                                }
                            }
                            solix_compilation::phases::parser::statements::declarations::Declaration::Enum(enum_declaration) => {
                                if enum_declaration.modifiers.contains(&solix_compilation::phases::parser::statements::declarations::Modifier::Public) {
                                    let enum_name = get_span_text_workspace(&snapshot.source_text, enum_declaration.name_span);
                                    let detail_string = format!("enum {}", enum_name);
                                    built_index
                                        .symbols_by_namespace
                                        .entry(package_path.clone())
                                        .or_default()
                                        .push((
                                            enum_name.to_string(),
                                            CompletionItemKind::ENUM,
                                            detail_string,
                                        ));
                                }
                            }
                            solix_compilation::phases::parser::statements::declarations::Declaration::Method(method_declaration) => {
                                if method_declaration.modifiers.contains(&solix_compilation::phases::parser::statements::declarations::Modifier::Public) {
                                    let method_name = get_span_text_workspace(&snapshot.source_text, method_declaration.name_span);
                                    let detail_string = format!("fn {}(...)", method_name);
                                    built_index
                                        .symbols_by_namespace
                                        .entry(package_path.clone())
                                        .or_default()
                                        .push((
                                            method_name.to_string(),
                                            CompletionItemKind::FUNCTION,
                                            detail_string,
                                        ));
                                }
                            }
                            solix_compilation::phases::parser::statements::declarations::Declaration::Field(field_declaration)
                                if field_declaration.modifiers.contains(&solix_compilation::phases::parser::statements::declarations::Modifier::Public)
                                    || field_declaration.modifiers.contains(&solix_compilation::phases::parser::statements::declarations::Modifier::Const) =>
                            {
                                let field_name = get_span_text_workspace(&snapshot.source_text, field_declaration.name_span);
                                let detail_string = format!("field {}", field_name);
                                built_index
                                    .symbols_by_namespace
                                    .entry(package_path.clone())
                                    .or_default()
                                    .push((
                                        field_name.to_string(),
                                        CompletionItemKind::VARIABLE,
                                        detail_string,
                                    ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Deduplicate symbols per namespace
        for symbol_list in built_index.symbols_by_namespace.values_mut() {
            let mut unique_names = std::collections::HashSet::new();
            symbol_list.retain(|(name, _, _)| unique_names.insert(name.clone()));
            symbol_list.sort_by(|left, right| left.0.cmp(&right.0));
        }

        // Cache and return
        if let Ok(mut guard) = self.package_export_index.write() {
            guard.insert(
                target_package_name.to_string(),
                (cache_key, built_index.clone()),
            );
        }

        built_index
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
        self.workspace_root
            .read()
            .ok()
            .and_then(|guard| guard.clone())
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

    /// Updates the in-memory document buffer immediately without running a blocking compilation pass.
    pub fn update_document(&self, uri: &Url, version: i32, content: String) {
        self.documents.insert(
            uri.clone(),
            VersionedDocument {
                uri: uri.clone(),
                version,
                content,
            },
        );
    }

    /// Performs an analysis pass for an updated document.
    pub fn analyze_document_change(
        &self,
        uri: &Url,
        version: i32,
        content: &str,
    ) -> AnalysisSnapshot {
        self.analyze_document(uri, version, content)
    }

    /// Updates an in-memory document buffer and performs an analysis pass.
    pub fn did_change(&self, uri: &Url, version: i32, content: String) -> AnalysisSnapshot {
        self.update_document(uri, version, content.clone());
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

    /// Returns the latest analysis snapshot overlaid with the live document buffer.
    ///
    /// If the live in-memory editor buffer contains text that differs from the last
    /// compiled snapshot (e.g. while debouncing), overlays the live text and re-lexes
    /// tokens synchronously while preserving the last good semantic bindings.
    #[must_use]
    pub fn get_live_snapshot(&self, uri: &Url) -> Option<AnalysisSnapshot> {
        let live_doc = self.documents.get(uri).map(|entry| entry.value().clone());
        let cached_snapshot = self.snapshots.get(uri).map(|entry| entry.value().clone());

        match (live_doc, cached_snapshot) {
            (Some(document), Some(mut snapshot)) => {
                if document.content != snapshot.source_text {
                    let (tokens, _) = tokenize_source(snapshot.source_id, &document.content);
                    snapshot.version = document.version;
                    snapshot.source_text = document.content;
                    snapshot.tokens = tokens;
                }
                Some(snapshot)
            }
            (Some(document), None) => {
                Some(self.analyze_document_change(uri, document.version, &document.content))
            }
            (None, Some(snapshot)) => Some(snapshot),
            (None, None) => None,
        }
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

    /// Searches upward from the target file path for an enclosing `manifest.yaml` (or `manifest.toml`).
    #[must_use]
    pub fn find_project_manifest(file_path: &Path) -> Option<PathBuf> {
        let mut current_dir = if file_path.is_file() {
            file_path.parent()?
        } else {
            file_path
        };

        loop {
            if let Some(found_manifest) = solix_common::project::find_manifest_path(current_dir) {
                return Some(found_manifest);
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
    fn analyze_document(
        &self,
        target_uri: &Url,
        version: i32,
        current_content: &str,
    ) -> AnalysisSnapshot {
        let target_path = target_uri.to_file_path().ok();
        let manifest_path = target_path
            .as_ref()
            .and_then(|path| Self::find_project_manifest(path))
            .or_else(|| {
                self.workspace_root()
                    .and_then(|root| Self::find_project_manifest(&root))
            });

        let (sources_map, target_source_name) = if let Some(ref manifest) = manifest_path {
            let project_root = manifest.parent().unwrap_or_else(|| Path::new("."));
            let discovered_files = Self::discover_project_sources(project_root);
            let mut map = HashMap::new();

            let normalized_target_path = target_path
                .as_ref()
                .map(|path| solix_common::strip_windows_verbatim_prefix(path.clone()));

            for file_path in discovered_files {
                let normalized_file_path = solix_common::strip_windows_verbatim_prefix(file_path);
                let file_url = Url::from_file_path(&normalized_file_path).ok();
                let file_name = normalized_file_path.to_string_lossy().to_string();

                let is_target = normalized_target_path
                    .as_ref()
                    .is_some_and(|target| target == &normalized_file_path)
                    || file_url.as_ref() == Some(target_uri);

                let file_content = if is_target {
                    current_content.to_string()
                } else if let Some(open_doc) =
                    file_url.as_ref().and_then(|url| self.documents.get(url))
                {
                    open_doc.content.clone()
                } else {
                    std::fs::read_to_string(&normalized_file_path).unwrap_or_default()
                };

                map.insert(file_name, file_content);
            }

            let target_name = normalized_target_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| target_uri.to_string());

            if !map.contains_key(&target_name) {
                map.insert(target_name.clone(), current_content.to_string());
            }

            (map, target_name)
        } else {
            let mut map = HashMap::new();
            let target_name = target_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| target_uri.to_string());
            map.insert(target_name.clone(), current_content.to_string());
            (map, target_name)
        };

        let mut dependency_specs = Vec::new();
        if let Some(ref manifest_file_path) = manifest_path
            && let Ok(project_manifest) =
                solix_common::project::ProjectManifest::from_file(manifest_file_path)
        {
            for resolved_dependency in project_manifest.resolved_dependencies() {
                if let Ok(parsed_version) =
                    solix_common::solver::PackageVersion::parse(&resolved_dependency.version)
                {
                    let mut requirement = solix_common::solver::DependencyRequirement::new(
                        resolved_dependency.package_name,
                        parsed_version,
                    );
                    if !resolved_dependency.build_profile.is_empty() {
                        requirement = requirement.with_profile(resolved_dependency.build_profile);
                    }
                    dependency_specs.push(requirement);
                }
            }
        }

        // Pre-populate dependency_binaries from in-memory cache to avoid disk reads
        let mut cached_binaries = Vec::new();
        for dependency_spec in &dependency_specs {
            if let Some(cached_binary) = self.cached_dependencies.get(&dependency_spec.name) {
                cached_binaries.push(cached_binary.value().clone());
            }
        }

        // Construct CompilationContext
        let mut options = CompilationOptions {
            profile: BuildProfile::Debug,
            codes: sources_map,
            dependency_specs,
            dependency_binaries: cached_binaries,
            ..Default::default()
        };
        if let Some(solver) = &self.solver {
            options = options.with_solver(solver.clone());
        }

        let mut context = CompilationContext::new(options);

        // Pre-run dependency resolution phase to ingest manifest dependencies
        let _ = solix_compilation::phases::dependency::run_dependency_resolution(&mut context);

        // Cache any newly resolved dependency binaries in memory
        for binary_program in &context.options.dependency_binaries {
            if !binary_program.package_name.is_empty() {
                self.cached_dependencies
                    .insert(binary_program.package_name.clone(), binary_program.clone());
            }
        }

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

        // Stage 2: Resilient parsing per source
        let current_sources_count = context.sources.len();
        if context.ast.len() < current_sources_count {
            context.ast.resize_with(current_sources_count, || None);
        }

        let mut target_parse_errors = Vec::new();
        for source_index in 0..current_sources_count {
            let source_id = source_index as u16;
            let source_text = &context.sources[source_index];
            let tokens: &[Token] = context
                .tokens
                .get(source_index)
                .map(|tok_vec| tok_vec.as_slice())
                .unwrap_or(&[]);

            let (parsed_ast, parse_errors) =
                solix_compilation::phases::parser::parse_source_resilient(
                    source_id,
                    tokens,
                    source_text,
                );
            context.ast[source_index] = Some(parsed_ast);
            if source_id == target_source_id {
                target_parse_errors = parse_errors;
            }
        }

        let target_source_text = context.sources[target_source_id as usize].clone();
        for error in &target_parse_errors {
            let error_span = error.source_span();
            if error_span.offset() < target_source_text.len() {
                accumulated_diagnostics
                    .push(parsing_error_to_diagnostic(&target_source_text, error));
            }
        }

        // Stage 2.5: Resilient extern import and standard library auto-resolution
        let mut resolved_packages = std::collections::HashSet::new();
        let mut ast_cursor = 0;
        while ast_cursor < context.ast.len() {
            let mut new_extern_imports = Vec::new();
            if let Some(ast) = &context.ast[ast_cursor] {
                for import_statement in &ast.imports {
                    let first_segment = if let Some(first_seg_span) = import_statement.path.first()
                    {
                        let span = *first_seg_span;
                        let text = &context.sources[ast_cursor];
                        if (span.end as usize) <= text.len() {
                            text[span.start as usize..span.end as usize].to_string()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    let should_resolve = import_statement.is_extern
                        && !first_segment.is_empty()
                        && !context.loaded_api_packages.contains(&first_segment)
                        && resolved_packages.insert(first_segment.clone());

                    if should_resolve {
                        new_extern_imports.push((first_segment, import_statement.span));
                    }
                }
            }
            ast_cursor += 1;

            for (package_name, import_span) in new_extern_imports {
                let _ = solix_compilation::phases::dependency::resolve_extern_import(
                    &package_name,
                    import_span,
                    &mut context,
                );
            }
        }

        // Stage 3: Semantic binding
        let _ = bind_context(&mut context);
        for (source_id, error) in &context.binding_errors {
            if *source_id == target_source_id {
                accumulated_diagnostics
                    .push(binding_error_to_diagnostic(&target_source_text, error));
            }
        }

        // Extract semantic artifacts
        let previous_semantic = self
            .snapshots
            .get(target_uri)
            .and_then(|snap| snap.last_good_semantic.clone());

        let new_semantic_snapshot = context.binding_artifacts.and_then(|artifacts| {
            let user_symbols_count = artifacts
                .symbol_table
                .symbols
                .iter()
                .filter(|symbol| symbol.source_id == target_source_id)
                .count();

            if !target_parse_errors.is_empty()
                && let Some(ref prev_sem) = previous_semantic
            {
                let prev_user_symbols_count = prev_sem
                    .symbol_table
                    .symbols
                    .iter()
                    .filter(|symbol| symbol.source_id == target_source_id)
                    .count();

                if user_symbols_count < prev_user_symbols_count {
                    return None;
                }
            }

            Some(Arc::new(BoundSemanticSnapshot {
                symbol_table: artifacts.symbol_table,
                layouts: artifacts.layouts,
                node_types: artifacts.node_types,
                resolved_symbols: artifacts.resolved_symbols,
                monomorphized_classes: artifacts.monomorphized_classes,
            }))
        });

        // Retain last good semantic snapshot if the current bind failed to produce one
        let last_good_semantic = new_semantic_snapshot.or(previous_semantic);

        // Stage 4: Dead code & unused symbol diagnostics
        let target_ast = context
            .ast
            .get(target_source_id as usize)
            .and_then(|opt| opt.clone());
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

        self.snapshots
            .insert(target_uri.clone(), final_snapshot.clone());
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

fn split_symbol_namespace(raw_symbol: &str, fallback_package: &str) -> (String, String) {
    if let Some(last_double_colon) = raw_symbol.rfind("::") {
        let namespace_part = &raw_symbol[..last_double_colon];
        let symbol_name = &raw_symbol[last_double_colon + 2..];
        let dot_namespace = namespace_part.replace("::", ".");
        (dot_namespace, symbol_name.to_string())
    } else {
        (fallback_package.to_string(), raw_symbol.to_string())
    }
}

fn insert_namespace_and_ancestors(
    namespaces: &mut std::collections::BTreeSet<String>,
    full_namespace: &str,
) {
    if full_namespace.is_empty() {
        return;
    }
    namespaces.insert(full_namespace.to_string());
    let segments: Vec<&str> = full_namespace.split('.').collect();
    for length in 1..segments.len() {
        let prefix = segments[..length].join(".");
        namespaces.insert(prefix);
    }
}

fn get_span_text_workspace(source: &str, span: Span) -> &str {
    let start_offset = span.start as usize;
    let end_offset = span.end as usize;
    if start_offset <= end_offset
        && end_offset <= source.len()
        && source.is_char_boundary(start_offset)
        && source.is_char_boundary(end_offset)
    {
        &source[start_offset..end_offset]
    } else {
        ""
    }
}
