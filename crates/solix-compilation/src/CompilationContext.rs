//! Compilation context and options managing multi-file sources and compilation pipeline artifacts.

#![forbid(unsafe_code)]

use crate::phases::lexing::tokens::{Span, Token};
use crate::phases::parser::statements::ProgramAst;
use solix_common::{LexingError, ParsingError};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

/// Configuration options provided to the compilation driver.
#[derive(Debug, Default)]
pub struct CompilationOptions {
    /// Mapping of source identifiers/paths to their source code contents.
    pub codes: HashMap<String, String>,
    /// Package or library name.
    pub package_name: String,
    /// Package semantic version.
    pub package_version: String,
    /// Human-readable library description.
    pub package_description: String,
    /// Optional customized compilation pipeline. If None, defaults to [`crate::pipeline::CompilationPipeline::standard()`].
    pub pipeline: Option<crate::pipeline::CompilationPipeline>,
    /// Whether to automatically include built-in standard library definitions (`blt.io`).
    pub include_builtins: bool,
}

impl CompilationOptions {
    /// Creates a new empty compilation options set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
            package_name: String::new(),
            package_version: "0.1.0".to_string(),
            package_description: String::new(),
            pipeline: None,
            include_builtins: false,
        }
    }

    /// Sets whether to include the standard built-in libraries (`blt.io`).
    #[must_use]
    pub fn with_builtins(mut self, enabled: bool) -> Self {
        self.include_builtins = enabled;
        self
    }

    /// Appends an in-memory source file to the compilation options.
    #[must_use]
    pub fn with_source(mut self, name: impl Into<String>, code: impl Into<String>) -> Self {
        self.codes.insert(name.into(), code.into());
        self
    }

    /// Sets package metadata for this compilation unit.
    #[must_use]
    pub fn with_package_metadata(
        mut self,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.package_name = name.into();
        self.package_version = version.into();
        self.package_description = description.into();
        self
    }

    /// Sets a customized compilation pipeline for this compilation context.
    #[must_use]
    pub fn with_pipeline(mut self, pipeline: crate::pipeline::CompilationPipeline) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Appends a custom compilation phase to the pipeline.
    #[must_use]
    pub fn with_phase<P: crate::phases::phase::CompilationPhase + 'static>(
        mut self,
        phase: P,
    ) -> Self {
        let mut pipeline = self.pipeline.unwrap_or_default();
        pipeline = pipeline.add_phase(phase);
        self.pipeline = Some(pipeline);
        self
    }
}

/// Global compilation state tracking registered source files and pipeline artifacts.
#[derive(Debug)]
pub struct CompilationContext {
    /// User-supplied compilation options.
    pub options: CompilationOptions,
    /// List of registered source file names/paths indexed by `source_id`.
    pub source_names: Vec<String>,
    /// List of source file contents indexed by `source_id`.
    pub sources: Vec<String>,
    /// Accumulated token streams produced by the lexer, indexed by `source_id`.
    pub tokens: Vec<Vec<Token>>,
    /// Abstract Syntax Trees produced by the parser, indexed by `source_id`.
    pub ast: Vec<Option<ProgramAst>>,
    /// Forward dependency graph: `dependencies[source_id]` contains `source_id`s that `source_id` imports.
    pub dependencies: Vec<Vec<u16>>,
    /// Reverse dependency graph: `dependents[source_id]` contains `source_id`s that import `source_id`.
    pub dependents: Vec<Vec<u16>>,
    /// Semantic analysis artifacts (symbol table, layouts, expression types), populated by the binder.
    pub binding_artifacts: Option<crate::phases::binder::BindingArtifacts>,
    /// Bytecode emission artifacts (per-source compiled functions and SLXB binary program).
    pub emission_artifacts: Option<crate::phases::emitter::pipeline::EmissionArtifacts>,
    /// Generic type-map blackboard for arbitrary phase and plugin state.
    pub extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl CompilationContext {
    /// Initializes a new compilation context from the provided options.
    #[must_use]
    pub fn new(mut options: CompilationOptions) -> Self {
        if options.include_builtins
            && !options
                .codes
                .contains_key(crate::builtins::BLT_IO_FILE_NAME)
        {
            options.codes.insert(
                crate::builtins::BLT_IO_FILE_NAME.to_string(),
                crate::builtins::BLT_IO_SOURCE.to_string(),
            );
        }

        let capacity = options.codes.len();
        let mut source_names = Vec::with_capacity(capacity);
        let mut sources = Vec::with_capacity(capacity);
        let mut tokens = Vec::with_capacity(capacity);
        let mut ast = Vec::with_capacity(capacity);
        let mut dependencies = Vec::with_capacity(capacity);
        let mut dependents = Vec::with_capacity(capacity);

        // Sort keys for deterministic source_id indexing
        let mut sorted_keys: Vec<_> = options.codes.keys().cloned().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            if let Some(code) = options.codes.get(&key) {
                source_names.push(key);
                sources.push(code.clone());
                tokens.push(Vec::new());
                ast.push(None);
                dependencies.push(Vec::new());
                dependents.push(Vec::new());
            }
        }

        Self {
            options,
            source_names,
            sources,
            tokens,
            ast,
            dependencies,
            dependents,
            binding_artifacts: None,
            emission_artifacts: None,
            extensions: HashMap::new(),
        }
    }

    /// Registers an additional source file dynamically, returning its assigned `source_id`.
    pub fn add_source(&mut self, name: impl Into<String>, code: impl Into<String>) -> u16 {
        let source_id = self.sources.len() as u16;
        self.source_names.push(name.into());
        self.sources.push(code.into());
        self.tokens.push(Vec::new());
        self.ast.push(None);
        self.dependencies.push(Vec::new());
        self.dependents.push(Vec::new());
        source_id
    }

    /// Resolves the exact string slice from the source code corresponding to a token span.
    #[must_use]
    pub fn get_lexeme(&self, span: Span) -> &str {
        let source = &self.sources[span.id as usize];
        &source[span.start as usize..span.end as usize]
    }

    /// Returns the source file name/path associated with a given `source_id`.
    #[must_use]
    pub fn get_source_name(&self, source_id: u16) -> Option<&str> {
        self.source_names
            .get(source_id as usize)
            .map(|s| s.as_str())
    }

    /// Resolves the numeric `source_id` associated with a registered source name or path.
    #[must_use]
    pub fn get_source_id(&self, name: &str) -> Option<u16> {
        self.source_names
            .iter()
            .position(|s| s == name)
            .map(|idx| idx as u16)
    }

    /// Returns the raw source code text associated with a given `source_id`.
    #[must_use]
    pub fn get_source_code(&self, source_id: u16) -> Option<&str> {
        self.sources.get(source_id as usize).map(|s| s.as_str())
    }

    /// Updates the source text of an existing source file and invalidates its cached artifacts.
    pub fn update_source(&mut self, source_id: u16, new_code: impl Into<String>) {
        let index = source_id as usize;
        if index < self.sources.len() {
            self.sources[index] = new_code.into();
            self.tokens[index].clear();
            self.ast[index] = None;
            self.binding_artifacts = None;
            self.emission_artifacts = None;
        }
    }

    /// Inserts a typed extension into the context blackboard.
    pub fn insert_extension<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Retrieves an immutable reference to a typed extension from the blackboard.
    #[must_use]
    pub fn get_extension<T: 'static>(&self) -> Option<&T> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Retrieves a mutable reference to a typed extension from the blackboard.
    pub fn get_extension_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.extensions
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Removes and returns a typed extension from the blackboard.
    pub fn remove_extension<T: 'static>(&mut self) -> Option<T> {
        self.extensions
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Executes the compilation pipeline (either configured in options or the default standard pipeline).
    ///
    /// # Errors
    /// Returns accumulated [`solix_common::CompilationDiagnostic`]s if any phase fails and halts.
    pub fn execute(&mut self) -> Result<(), Vec<solix_common::CompilationDiagnostic>> {
        let mut pipeline = self.options.pipeline.take().unwrap_or_default();
        let result = pipeline.execute(self);
        self.options.pipeline = Some(pipeline);
        result
    }

    /// Executes a specific compilation pipeline against this context.
    ///
    /// # Errors
    /// Returns accumulated [`solix_common::CompilationDiagnostic`]s if any phase fails and halts.
    pub fn execute_pipeline(
        &mut self,
        pipeline: &mut crate::pipeline::CompilationPipeline,
    ) -> Result<(), Vec<solix_common::CompilationDiagnostic>> {
        pipeline.execute(self)
    }

    /// Executes full bytecode emission across all sources, returning the emitted artifacts.
    ///
    /// # Errors
    /// Returns `EmissionError` if bytecode emission fails.
    pub fn emit_context(
        &mut self,
    ) -> Result<
        &crate::phases::emitter::pipeline::EmissionArtifacts,
        solix_common::diagnostics::EmissionError,
    > {
        if self.emission_artifacts.is_none() {
            let artifacts = crate::phases::emitter::pipeline::emit_context(self)?;
            self.emission_artifacts = Some(artifacts);
        }
        Ok(self.emission_artifacts.as_ref().unwrap())
    }

    /// Returns the compiled functions for a specific source file index.
    ///
    /// # Errors
    /// Returns `EmissionError` if bytecode emission fails.
    pub fn emit_source(
        &mut self,
        source_id: u16,
    ) -> Result<&[solix_common::bytecode::CompiledFunction], solix_common::diagnostics::EmissionError>
    {
        if self.emission_artifacts.is_none() {
            self.emit_context()?;
        }
        let artifacts = self.emission_artifacts.as_ref().unwrap();
        let index = source_id as usize;
        if index < artifacts.functions_by_source.len() {
            Ok(&artifacts.functions_by_source[index])
        } else {
            Ok(&[])
        }
    }

    /// Incrementally tokenizes a single source file in the context.
    ///
    /// # Errors
    /// Returns lexing errors if unrecognized tokens were encountered in the specified source file.
    pub fn lex_source(&mut self, source_id: u16) -> Result<(), Vec<LexingError>> {
        let index = source_id as usize;
        let source_text = &self.sources[index];
        match crate::phases::lexing::lex_source(source_id, source_text) {
            Ok(tokens) => {
                self.tokens[index] = tokens;
                Ok(())
            }
            Err(errors) => {
                self.tokens[index].clear();
                Err(errors)
            }
        }
    }

    /// Incrementally parses a single source file in the context.
    ///
    /// # Errors
    /// Returns parsing errors if syntax violations were encountered in the specified source file.
    pub fn parse_source(&mut self, source_id: u16) -> Result<(), Vec<ParsingError>> {
        let index = source_id as usize;
        let source_text = &self.sources[index];
        let tokens = &self.tokens[index];
        match crate::phases::parser::parse_source(source_id, tokens, source_text) {
            Ok(program_ast) => {
                self.ast[index] = Some(program_ast);
                Ok(())
            }
            Err(errors) => {
                self.ast[index] = None;
                Err(errors)
            }
        }
    }

    /// Retrieves a reference to the parsed AST for a given `source_id`, if available.
    #[must_use]
    pub fn get_ast(&self, source_id: u16) -> Option<&ProgramAst> {
        self.ast
            .get(source_id as usize)
            .and_then(|opt| opt.as_ref())
    }

    /// Returns an iterator yielding references to all successfully parsed [`ProgramAst`]s in the context.
    pub fn valid_asts(&self) -> impl Iterator<Item = &ProgramAst> {
        self.ast.iter().filter_map(|opt| opt.as_ref())
    }

    /// Registers a dependency edge indicating that `from_source` imports/depends on `to_source`.
    pub fn register_dependency(&mut self, from_source: u16, to_source: u16) {
        let from_idx = from_source as usize;
        let to_idx = to_source as usize;
        if from_idx < self.dependencies.len() && to_idx < self.dependents.len() {
            if !self.dependencies[from_idx].contains(&to_source) {
                self.dependencies[from_idx].push(to_source);
            }
            if !self.dependents[to_idx].contains(&from_source) {
                self.dependents[to_idx].push(from_source);
            }
        }
    }

    /// Clears outbound dependency edges originating from `from_source`.
    pub fn clear_dependencies_for(&mut self, from_source: u16) {
        let from_idx = from_source as usize;
        if from_idx < self.dependencies.len() {
            let previous_deps = std::mem::take(&mut self.dependencies[from_idx]);
            for target_id in previous_deps {
                let target_idx = target_id as usize;
                if target_idx < self.dependents.len() {
                    self.dependents[target_idx].retain(|&id| id != from_source);
                }
            }
        }
    }

    /// Returns the immediate dependent source IDs that import `source_id`.
    #[must_use]
    pub fn get_dependents(&self, source_id: u16) -> &[u16] {
        self.dependents
            .get(source_id as usize)
            .map(|deps| deps.as_slice())
            .unwrap_or(&[])
    }

    /// Computes the transitive topological closure of all sources that directly or indirectly
    /// depend on `source_id` (the set of sources that must be rebound when `source_id` changes).
    #[must_use]
    pub fn get_transitive_dependents(&self, source_id: u16) -> Vec<u16> {
        let mut visited = HashSet::new();
        let mut queue = vec![source_id];

        while let Some(current) = queue.pop() {
            for &dependent in self.get_dependents(current) {
                if visited.insert(dependent) {
                    queue.push(dependent);
                }
            }
        }

        let mut result: Vec<u16> = visited.into_iter().collect();
        result.sort_unstable();
        result
    }
}

/// Unified error enum representing failure at any stage of the compilation pipeline.
#[derive(Debug, thiserror::Error)]
pub enum CompilationError {
    #[error("Lexical analysis error")]
    LexingError,
    #[error("Syntax parsing error")]
    ParsingError,
    #[error("Semantic binding error")]
    BindingError,
    #[error("Bytecode emission error")]
    EmitterError,
}
