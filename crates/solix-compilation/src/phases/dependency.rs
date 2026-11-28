//! Phase 0: Dependency resolution and synthetic API extraction phase.
//!
//! Executes before lexical analysis to resolve all explicit (`--deps`) and pre-loaded
//! dependencies, verify Semantic Versioning compatibility across the entire dependency graph,
//! and synthesize API interfaces (`manifest_to_source`) directly into the compilation context.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::diagnostics::{BindingError, CompilationDiagnostic};
use crate::manifest::manifest_to_source;
use crate::phases::lexing::Span;
use crate::phases::phase::CompilationPhase;
use miette::SourceSpan;
use solix_common::bytecode::BinaryProgram;
use solix_common::solver::{DependencyRequirement, PackageVersion, SolverError};
use std::collections::{HashMap, VecDeque};

/// Dependency resolution compilation phase running before lexical analysis.
#[derive(Debug, Default, Clone)]
pub struct DependencyResolutionPhase;

impl DependencyResolutionPhase {
    /// Creates a new [`DependencyResolutionPhase`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl CompilationPhase for DependencyResolutionPhase {
    fn name(&self) -> &'static str {
        "Dependency Resolution"
    }

    fn run(&mut self, context: &mut CompilationContext) -> Result<(), Vec<CompilationDiagnostic>> {
        run_dependency_resolution(context)
    }
}

/// Executes pre-lexing dependency resolution and synthetic API extraction.
pub fn run_dependency_resolution(
    context: &mut CompilationContext,
) -> Result<(), Vec<CompilationDiagnostic>> {
    let mut work_queue: VecDeque<(DependencyRequirement, String)> = VecDeque::new();
    let mut solved_map: HashMap<String, (PackageVersion, BinaryProgram, Option<String>)> =
        HashMap::new();
    let mut accumulated_diagnostics = Vec::new();

    // 1. Enqueue pre-loaded binary programs
    for (index, binary_program) in context.options.dependency_binaries.iter().enumerate() {
        let package_name = if !binary_program.package_name.is_empty() {
            binary_program.package_name.clone()
        } else if !binary_program.export_manifest.package_name.is_empty() {
            binary_program.export_manifest.package_name.clone()
        } else {
            format!("lib_{index}")
        };
        let raw_version = if !binary_program.package_version.is_empty() {
            &binary_program.package_version
        } else {
            &binary_program.export_manifest.package_version
        };
        let parsed_version =
            PackageVersion::parse(raw_version).unwrap_or_else(|_| PackageVersion::new(0, 1, 0));
        let preloaded_profile = (!binary_program.build_profile.is_empty())
            .then(|| binary_program.build_profile.clone());
        solved_map.insert(
            package_name.clone(),
            (parsed_version, binary_program.clone(), preloaded_profile),
        );
        for transitive_requirement in &binary_program.export_manifest.dependencies {
            work_queue.push_back((transitive_requirement.clone(), package_name.clone()));
        }
    }

    // 2. Enqueue command-line explicit dependencies
    for dependency_spec in &context.options.dependency_specs {
        work_queue.push_back((dependency_spec.clone(), "compiler options".to_string()));
    }

    // 3. Resolve the transitive requirement queue
    while let Some((requirement, requester_name)) = work_queue.pop_front() {
        if let Some((existing_version, existing_program, existing_profile)) =
            solved_map.get_mut(&requirement.name)
        {
            if existing_version.major != requirement.version.major {
                accumulated_diagnostics.push(CompilationDiagnostic::from(
                    BindingError::IncompatibleDependencyVersion {
                        name: requirement.name.clone(),
                        reason: format!(
                            "Conflicting major version: required '{}' (requested by '{}'), but already resolved to '{}'",
                            requirement.version, requester_name, existing_version
                        ),
                        span: SourceSpan::from(0..0),
                    },
                ));
                continue;
            }

            if context.options.verify_checksums && !requirement.sha256.is_empty() {
                let actual_sha256 = existing_program.compute_sha256();
                if requirement.sha256 != actual_sha256 {
                    accumulated_diagnostics.push(CompilationDiagnostic::from(
                        BindingError::DependencyChecksumMismatch {
                            name: requirement.name.clone(),
                            version: requirement.version.to_string(),
                            expected_sha256: requirement.sha256.clone(),
                            actual_sha256,
                            span: SourceSpan::from(0..0),
                        },
                    ));
                    continue;
                }
            }

            if *existing_version < requirement.version {
                if let Some(solver) = &context.solver {
                    let lookup_name = if let Some(ref target_profile) = requirement.profile {
                        format!("{}-{}", requirement.name, target_profile)
                    } else {
                        requirement.name.clone()
                    };
                    match solver.resolve_compatible(&lookup_name, requirement.version) {
                        Ok(upgraded_package) => {
                            *existing_version = upgraded_package.version;
                            *existing_program = upgraded_package.program.clone();
                            *existing_profile = requirement.profile.clone();
                            for transitive in &upgraded_package.program.export_manifest.dependencies
                            {
                                work_queue
                                    .push_back((transitive.clone(), requirement.name.clone()));
                            }
                        }
                        Err(solver_error) => {
                            accumulated_diagnostics.push(CompilationDiagnostic::from(
                                BindingError::IncompatibleDependencyVersion {
                                    name: requirement.name.clone(),
                                    reason: format!(
                                        "Cannot satisfy requirement '{}' (requested by '{}'): {solver_error}",
                                        requirement.version, requester_name
                                    ),
                                    span: SourceSpan::from(0..0),
                                },
                            ));
                        }
                    }
                } else {
                    accumulated_diagnostics.push(CompilationDiagnostic::from(
                        BindingError::IncompatibleDependencyVersion {
                            name: requirement.name.clone(),
                            reason: format!(
                                "Existing version '{}' is lower than required '{}' (requested by '{}') and no solver is configured",
                                existing_version, requirement.version, requester_name
                            ),
                            span: SourceSpan::from(0..0),
                        },
                    ));
                }
            }
        } else if let Some(solver) = &context.solver {
            let lookup_name = if let Some(ref target_profile) = requirement.profile {
                format!("{}-{}", requirement.name, target_profile)
            } else {
                requirement.name.clone()
            };
            match solver.resolve_compatible(&lookup_name, requirement.version) {
                Ok(solved_package) => {
                    let program = solved_package.program;
                    let version = solved_package.version;

                    if context.options.verify_checksums && !requirement.sha256.is_empty() {
                        let actual_sha256 = program.compute_sha256();
                        if requirement.sha256 != actual_sha256 {
                            accumulated_diagnostics.push(CompilationDiagnostic::from(
                                BindingError::DependencyChecksumMismatch {
                                    name: requirement.name.clone(),
                                    version: requirement.version.to_string(),
                                    expected_sha256: requirement.sha256.clone(),
                                    actual_sha256,
                                    span: SourceSpan::from(0..0),
                                },
                            ));
                            continue;
                        }
                    }

                    for transitive in &program.export_manifest.dependencies {
                        work_queue.push_back((transitive.clone(), requirement.name.clone()));
                    }
                    solved_map.insert(
                        requirement.name.clone(),
                        (version, program, requirement.profile.clone()),
                    );
                }
                Err(SolverError::PackageNotFound { .. }) => {
                    accumulated_diagnostics.push(CompilationDiagnostic::from(
                        BindingError::ExternalPackageNotFound {
                            name: requirement.name.clone(),
                            reason: format!(
                                "Package not found in package store (requested by '{requester_name}')"
                            ),
                            span: SourceSpan::from(0..0),
                        },
                    ));
                }
                Err(solver_error) => {
                    accumulated_diagnostics.push(CompilationDiagnostic::from(
                        BindingError::IncompatibleDependencyVersion {
                            name: requirement.name.clone(),
                            reason: format!(
                                "Failed to resolve package requirement '{}@{}': {solver_error}",
                                requirement.name, requirement.version
                            ),
                            span: SourceSpan::from(0..0),
                        },
                    ));
                }
            }
        } else {
            accumulated_diagnostics.push(CompilationDiagnostic::from(
                BindingError::ExternalPackageNotFound {
                    name: requirement.name.clone(),
                    reason: format!(
                        "Package requested by '{requester_name}', but no module solver configured"
                    ),
                    span: SourceSpan::from(0..0),
                },
            ));
        }
    }

    if !accumulated_diagnostics.is_empty() {
        return Err(accumulated_diagnostics);
    }

    // 4. Record into dependencies_map and synthesize API interfaces
    let mut sorted_packages: Vec<(String, PackageVersion, BinaryProgram, Option<String>)> =
        solved_map
            .into_iter()
            .map(|(package_name, (version, program, profile))| {
                (package_name, version, program, profile)
            })
            .collect();
    sorted_packages.sort_by(|package_a, package_b| package_a.0.cmp(&package_b.0));

    for (package_name, version, program, resolved_profile) in sorted_packages {
        let should_embed_hash = context.options.embed_dependency_hashes
            && context.options.link_mode != crate::CompilationContext::LinkMode::Static;
        let package_sha256 = if should_embed_hash {
            program.compute_sha256()
        } else {
            String::new()
        };
        let mut recorded_requirement = DependencyRequirement::new(package_name.clone(), version);
        if !package_sha256.is_empty() {
            recorded_requirement = recorded_requirement.with_sha256(package_sha256);
        }
        if let Some(profile_name) = resolved_profile {
            recorded_requirement = recorded_requirement.with_profile(profile_name);
        }
        context
            .dependencies_map
            .insert(package_name.clone(), recorded_requirement);

        if context.loaded_api_packages.insert(package_name.clone()) {
            ingest_package_api_source(
                &package_name,
                version,
                program,
                SourceSpan::from(0..0),
                context,
            )?;
        }
    }

    Ok(())
}

fn ingest_package_api_source(
    package_name: &str,
    version: PackageVersion,
    program: BinaryProgram,
    import_span: SourceSpan,
    context: &mut CompilationContext,
) -> Result<(), Vec<CompilationDiagnostic>> {
    let (source_name, source_code) = if !program.export_manifest.api_header.is_empty() {
        (
            format!("{package_name}@{version}.slx"),
            program.export_manifest.api_header.clone(),
        )
    } else {
        let manifest_json =
            serde_json::to_string(&program.export_manifest).map_err(|serialize_error| {
                vec![CompilationDiagnostic::from(
                    BindingError::ExternalPackageNotFound {
                        name: package_name.to_string(),
                        reason: format!("Failed to serialize export manifest: {serialize_error}"),
                        span: import_span,
                    },
                )]
            })?;

        manifest_to_source(&manifest_json).map_err(|synthesis_error| {
            vec![CompilationDiagnostic::from(
                BindingError::ExternalPackageNotFound {
                    name: package_name.to_string(),
                    reason: format!("Failed to synthesize API source: {synthesis_error}"),
                    span: import_span,
                },
            )]
        })?
    };

    let source_id = context
        .add_source(source_name, &source_code)
        .map_err(|diagnostic| vec![diagnostic])?;
    context.mark_external_source(source_id);

    let has_binary = context.options.dependency_binaries.iter().any(|existing| {
        !existing.package_name.is_empty() && existing.package_name == program.package_name
    });

    if !program.export_manifest.exported_ast_bytes.is_empty() {
        let deserialized_result = bincode::deserialize::<
            crate::phases::parser::statements::ProgramAst,
        >(&program.export_manifest.exported_ast_bytes)
        .or_else(|_| {
            serde_json::from_slice::<crate::phases::parser::statements::ProgramAst>(
                &program.export_manifest.exported_ast_bytes,
            )
        });

        if !has_binary {
            context.options.dependency_binaries.push(program);
        }

        if let Ok(mut deserialized_ast) = deserialized_result {
            deserialized_ast.source_id = source_id;
            selectively_prune_ast(&mut deserialized_ast, package_name, context, &source_code);
            context.ast[source_id as usize] = Some(deserialized_ast);
            return Ok(());
        }
    } else if !has_binary {
        context.options.dependency_binaries.push(program);
    }

    let (tokens, lex_errors) =
        crate::phases::lexing::lexer::tokenize_source(source_id, &source_code);
    if !lex_errors.is_empty() {
        let diagnostics = lex_errors
            .into_iter()
            .map(|error| CompilationDiagnostic::from(error).with_source_id(source_id))
            .collect();
        return Err(diagnostics);
    }
    context.tokens[source_id as usize] = tokens;

    let ast = crate::phases::parser::parse_source(
        source_id,
        &context.tokens[source_id as usize],
        &source_code,
    )
    .map_err(|parse_errors| {
        parse_errors
            .into_iter()
            .map(|error| CompilationDiagnostic::from(error).with_source_id(source_id))
            .collect::<Vec<_>>()
    })?;

    context.ast[source_id as usize] = Some(ast);
    Ok(())
}

/// Dynamically resolves an external package requirement during parsing (from `extern import`),
/// validates transitive dependencies, synthesizes the API source, lexes and parses it on the fly,
/// and adds it into the compilation context.
pub fn resolve_extern_import(
    package_name: &str,
    import_span: Span,
    context: &mut CompilationContext,
) -> Result<(), Vec<CompilationDiagnostic>> {
    if context.loaded_api_packages.contains(package_name) {
        return Ok(());
    }

    let solver = context.solver.clone().ok_or_else(|| {
        vec![CompilationDiagnostic::from(
            BindingError::ExternalPackageNotFound {
                name: package_name.to_string(),
                reason: "No module solver configured in compilation context".to_string(),
                span: import_span.to_source_span(),
            },
        )]
    })?;

    let solved_package = solver
        .resolve_latest(package_name)
        .map_err(|solver_error| match solver_error {
            SolverError::PackageNotFound { .. } => {
                vec![CompilationDiagnostic::from(
                    BindingError::ExternalPackageNotFound {
                        name: package_name.to_string(),
                        reason: "Package not found in package store".to_string(),
                        span: import_span.to_source_span(),
                    },
                )]
            }
            other_error => {
                vec![CompilationDiagnostic::from(
                    BindingError::IncompatibleDependencyVersion {
                        name: package_name.to_string(),
                        reason: format!(
                            "Failed to resolve package '{package_name}': {other_error}"
                        ),
                        span: import_span.to_source_span(),
                    },
                )]
            }
        })?;

    // Check transitive dependencies against existing dependencies_map
    for transitive_requirement in &solved_package.program.export_manifest.dependencies {
        if let Some(existing) = context.dependencies_map.get(&transitive_requirement.name) {
            if existing.version.major != transitive_requirement.version.major
                || existing.version < transitive_requirement.version
            {
                return Err(vec![CompilationDiagnostic::from(
                    BindingError::IncompatibleDependencyVersion {
                        name: transitive_requirement.name.clone(),
                        reason: format!(
                            "Transitive dependency conflict: required '{}@{}', but already resolved to '{}'",
                            transitive_requirement.name,
                            transitive_requirement.version,
                            existing.version
                        ),
                        span: import_span.to_source_span(),
                    },
                )]);
            }
        } else {
            // Recursively resolve missing transitive dependency
            resolve_transitive_requirement(transitive_requirement, import_span, context, &solver)?;
        }
    }

    // Record in dependencies_map
    let package_sha256 = solved_package.program.compute_sha256();
    context.dependencies_map.insert(
        package_name.to_string(),
        DependencyRequirement::new(package_name.to_string(), solved_package.version)
            .with_sha256(package_sha256),
    );

    // Synthesize API, lex, and parse into context
    if context.loaded_api_packages.insert(package_name.to_string()) {
        ingest_package_api_source(
            package_name,
            solved_package.version,
            solved_package.program,
            import_span.to_source_span(),
            context,
        )?;
    }

    Ok(())
}

fn resolve_transitive_requirement(
    requirement: &DependencyRequirement,
    import_span: Span,
    context: &mut CompilationContext,
    solver: &std::sync::Arc<dyn solix_common::solver::ModuleSolver>,
) -> Result<(), Vec<CompilationDiagnostic>> {
    let solved_package = solver
        .resolve_compatible(&requirement.name, requirement.version)
        .map_err(|solver_error| {
            vec![CompilationDiagnostic::from(
                BindingError::IncompatibleDependencyVersion {
                    name: requirement.name.clone(),
                    reason: format!(
                        "Failed to resolve transitive dependency '{}@{}': {solver_error}",
                        requirement.name, requirement.version
                    ),
                    span: import_span.to_source_span(),
                },
            )]
        })?;

    if context.options.verify_checksums && !requirement.sha256.is_empty() {
        let actual_sha256 = solved_package.program.compute_sha256();
        if requirement.sha256 != actual_sha256 {
            return Err(vec![CompilationDiagnostic::from(
                BindingError::DependencyChecksumMismatch {
                    name: requirement.name.clone(),
                    version: requirement.version.to_string(),
                    expected_sha256: requirement.sha256.clone(),
                    actual_sha256,
                    span: import_span.to_source_span(),
                },
            )]);
        }
    }

    // Check transitive sub-dependencies
    for child_requirement in &solved_package.program.export_manifest.dependencies {
        if let Some(existing) = context.dependencies_map.get(&child_requirement.name) {
            if existing.version.major != child_requirement.version.major
                || existing.version < child_requirement.version
            {
                return Err(vec![CompilationDiagnostic::from(
                    BindingError::IncompatibleDependencyVersion {
                        name: child_requirement.name.clone(),
                        reason: format!(
                            "Transitive dependency conflict: required '{}@{}', but already resolved to '{}'",
                            child_requirement.name, child_requirement.version, existing.version
                        ),
                        span: import_span.to_source_span(),
                    },
                )]);
            }
        } else {
            resolve_transitive_requirement(child_requirement, import_span, context, solver)?;
        }
    }

    let package_sha256 = solved_package.program.compute_sha256();
    context.dependencies_map.insert(
        requirement.name.clone(),
        DependencyRequirement::new(requirement.name.clone(), solved_package.version)
            .with_sha256(package_sha256),
    );

    if context.loaded_api_packages.insert(requirement.name.clone()) {
        ingest_package_api_source(
            &requirement.name,
            solved_package.version,
            solved_package.program,
            import_span.to_source_span(),
            context,
        )?;
    }

    Ok(())
}

fn get_span_slice(span: Span, source_text: &str) -> &str {
    let start_offset = span.start as usize;
    let end_offset = span.end as usize;
    if start_offset <= end_offset && end_offset <= source_text.len() {
        &source_text[start_offset..end_offset]
    } else {
        ""
    }
}

fn collect_type_names(
    annotation: &crate::phases::parser::statements::TypeAnnotation,
    source_text: &str,
    target_set: &mut std::collections::HashSet<String>,
) {
    use crate::phases::parser::statements::TypeAnnotation;
    match annotation {
        TypeAnnotation::Primitive { .. } => {}
        TypeAnnotation::Named { path, .. } => {
            if let Some(last_segment) = path.last() {
                let simple_name = get_span_slice(*last_segment, source_text);
                if !simple_name.is_empty() {
                    target_set.insert(simple_name.to_string());
                }
            }
            let segments: Vec<&str> = path
                .iter()
                .map(|span| get_span_slice(*span, source_text))
                .filter(|segment| !segment.is_empty())
                .collect();
            if !segments.is_empty() {
                target_set.insert(segments.join("::"));
            }
        }
        TypeAnnotation::Generic {
            base, arguments, ..
        } => {
            collect_type_names(base, source_text, target_set);
            for argument in arguments {
                collect_type_names(argument, source_text, target_set);
            }
        }
        TypeAnnotation::Array { element_type, .. } => {
            collect_type_names(element_type, source_text, target_set);
        }
        TypeAnnotation::Tuple { elements, .. } => {
            for element in elements {
                collect_type_names(element, source_text, target_set);
            }
        }
        TypeAnnotation::FunctionPointer {
            return_type,
            parameters,
            ..
        } => {
            collect_type_names(return_type, source_text, target_set);
            for parameter in parameters {
                collect_type_names(parameter, source_text, target_set);
            }
        }
    }
}

fn collect_identifiers_from_slice(slice: &str, target_set: &mut std::collections::HashSet<String>) {
    let bytes = slice.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            target_set.insert(slice[start..index].to_string());
        } else {
            index += 1;
        }
    }
}

fn collect_user_identifiers(
    context: &CompilationContext,
    target_set: &mut std::collections::HashSet<String>,
) {
    use crate::phases::lexing::TokenKind;
    for (source_idx, source_text) in context.sources.iter().enumerate() {
        if u16::try_from(source_idx).is_ok_and(|id| context.is_external_source(id)) {
            continue;
        }
        if let Some(tokens) = context.tokens.get(source_idx)
            && !tokens.is_empty()
        {
            for token in tokens {
                if token.kind == TokenKind::Identifier {
                    let lexeme = context.get_lexeme(token.span);
                    if !lexeme.is_empty() {
                        target_set.insert(lexeme.to_string());
                    }
                }
            }
        } else {
            collect_identifiers_from_slice(source_text, target_set);
        }
    }
}

fn scan_raw_source_imports(
    source_text: &str,
    package_name: &str,
    has_any_package_import: &mut bool,
    has_wildcard_import: &mut bool,
    imported_symbols: &mut std::collections::HashSet<String>,
) {
    for statement in source_text.split(';') {
        let trimmed = statement.trim();
        let rest = if let Some(after) = trimmed.strip_prefix("extern") {
            let after_ws = after.trim_start();
            if let Some(after_import) = after_ws.strip_prefix("import") {
                after_import.trim()
            } else {
                continue;
            }
        } else if let Some(after_import) = trimmed.strip_prefix("import") {
            after_import.trim()
        } else {
            continue;
        };

        let normalized = rest.replace("::", ".");
        let segments: Vec<&str> = normalized
            .split('.')
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.first().copied() != Some(package_name) {
            continue;
        }
        *has_any_package_import = true;
        if segments.len() == 1 || segments.last().copied() == Some("*") {
            *has_wildcard_import = true;
        } else if let Some(last_segment) = segments.last() {
            imported_symbols.insert(segments.join("::"));
            imported_symbols.insert((*last_segment).to_string());
        }
    }
}

fn selectively_prune_ast(
    ast: &mut crate::phases::parser::statements::ProgramAst,
    package_name: &str,
    context: &CompilationContext,
    source_text: &str,
) {
    use crate::phases::parser::statements::{ClassMember, Declaration};

    let mut has_wildcard_import = false;
    let imported_namespaces: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut imported_symbols = std::collections::HashSet::new();
    let mut has_any_package_import = false;

    for (source_idx, user_source_text) in context.sources.iter().enumerate() {
        if u16::try_from(source_idx).is_ok_and(|id| context.is_external_source(id)) {
            continue;
        }
        if let Some(Some(user_ast)) = context.ast.get(source_idx) {
            for import_statement in &user_ast.imports {
                if import_statement.path.is_empty() {
                    continue;
                }
                let first_lexeme = context.get_lexeme(import_statement.path[0]);
                if first_lexeme != package_name {
                    continue;
                }
                has_any_package_import = true;
                if import_statement.path.len() == 1 || import_statement.is_wildcard {
                    has_wildcard_import = true;
                } else {
                    let symbol_segments: Vec<String> = import_statement
                        .path
                        .iter()
                        .map(|span| context.get_lexeme(*span).to_string())
                        .collect();
                    imported_symbols.insert(symbol_segments.join("::"));
                    if let Some(last_segment) = symbol_segments.last() {
                        imported_symbols.insert(last_segment.clone());
                    }
                }
            }
        } else {
            scan_raw_source_imports(
                user_source_text,
                package_name,
                &mut has_any_package_import,
                &mut has_wildcard_import,
                &mut imported_symbols,
            );
        }
    }

    if !has_any_package_import {
        return;
    }

    let mut needed_symbols = imported_symbols;
    if has_wildcard_import {
        collect_user_identifiers(context, &mut needed_symbols);
    }
    needed_symbols.insert("String".to_string());
    needed_symbols.insert("IStringable".to_string());
    needed_symbols.insert("Object".to_string());

    let core_namespace_prefix = format!("{package_name}::core");

    loop {
        let size_before = needed_symbols.len();

        for declaration in &ast.declarations {
            match declaration {
                Declaration::Class(class_declaration) => {
                    let class_name = get_span_slice(class_declaration.name_span, source_text);
                    let namespace_segments: Vec<&str> = class_declaration
                        .namespace_path
                        .iter()
                        .map(|span| get_span_slice(*span, source_text))
                        .filter(|segment| !segment.is_empty())
                        .collect();
                    let namespace = namespace_segments.join("::");
                    let qualified_name = if namespace.is_empty() {
                        class_name.to_string()
                    } else {
                        format!("{namespace}::{class_name}")
                    };

                    let is_needed = needed_symbols.contains(class_name)
                        || needed_symbols.contains(&qualified_name)
                        || imported_namespaces.contains(&namespace)
                        || namespace.starts_with(&core_namespace_prefix);

                    if is_needed {
                        if let Some(super_class) = &class_declaration.super_class {
                            collect_type_names(super_class, source_text, &mut needed_symbols);
                        }
                        for interface_type in &class_declaration.interfaces {
                            collect_type_names(interface_type, source_text, &mut needed_symbols);
                        }
                        for member in &class_declaration.members {
                            match member {
                                ClassMember::Field(field_declaration) => {
                                    collect_type_names(
                                        &field_declaration.field_type,
                                        source_text,
                                        &mut needed_symbols,
                                    );
                                    if field_declaration.initializer.is_some() {
                                        let slice =
                                            get_span_slice(field_declaration.span, source_text);
                                        collect_identifiers_from_slice(slice, &mut needed_symbols);
                                    }
                                }
                                ClassMember::Method(method_declaration) => {
                                    collect_type_names(
                                        &method_declaration.return_type,
                                        source_text,
                                        &mut needed_symbols,
                                    );
                                    for parameter in &method_declaration.parameters {
                                        collect_type_names(
                                            &parameter.param_type,
                                            source_text,
                                            &mut needed_symbols,
                                        );
                                    }
                                    if let Some(body) = &method_declaration.body {
                                        let slice = get_span_slice(body.span, source_text);
                                        collect_identifiers_from_slice(slice, &mut needed_symbols);
                                    }
                                }
                                ClassMember::Constructor(ctor_declaration) => {
                                    for parameter in &ctor_declaration.parameters {
                                        collect_type_names(
                                            &parameter.param_type,
                                            source_text,
                                            &mut needed_symbols,
                                        );
                                    }
                                    let slice =
                                        get_span_slice(ctor_declaration.body.span, source_text);
                                    collect_identifiers_from_slice(slice, &mut needed_symbols);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Declaration::Interface(interface_declaration) => {
                    let iface_name = get_span_slice(interface_declaration.name_span, source_text);
                    let namespace_segments: Vec<&str> = interface_declaration
                        .namespace_path
                        .iter()
                        .map(|span| get_span_slice(*span, source_text))
                        .filter(|segment| !segment.is_empty())
                        .collect();
                    let namespace = namespace_segments.join("::");
                    let qualified_name = if namespace.is_empty() {
                        iface_name.to_string()
                    } else {
                        format!("{namespace}::{iface_name}")
                    };
                    if needed_symbols.contains(iface_name)
                        || needed_symbols.contains(&qualified_name)
                        || imported_namespaces.contains(&namespace)
                        || namespace.starts_with(&core_namespace_prefix)
                    {
                        for super_iface in &interface_declaration.super_interfaces {
                            collect_type_names(super_iface, source_text, &mut needed_symbols);
                        }
                        for method_declaration in &interface_declaration.methods {
                            collect_type_names(
                                &method_declaration.return_type,
                                source_text,
                                &mut needed_symbols,
                            );
                            for parameter in &method_declaration.parameters {
                                collect_type_names(
                                    &parameter.param_type,
                                    source_text,
                                    &mut needed_symbols,
                                );
                            }
                        }
                    }
                }
                Declaration::Method(method_declaration) => {
                    let method_name = get_span_slice(method_declaration.name_span, source_text);
                    let namespace_segments: Vec<&str> = method_declaration
                        .namespace_path
                        .iter()
                        .map(|span| get_span_slice(*span, source_text))
                        .filter(|segment| !segment.is_empty())
                        .collect();
                    let namespace = namespace_segments.join("::");
                    let qualified_name = if namespace.is_empty() {
                        method_name.to_string()
                    } else {
                        format!("{namespace}::{method_name}")
                    };
                    if needed_symbols.contains(method_name)
                        || needed_symbols.contains(&qualified_name)
                        || imported_namespaces.contains(&namespace)
                        || namespace.starts_with(&core_namespace_prefix)
                    {
                        collect_type_names(
                            &method_declaration.return_type,
                            source_text,
                            &mut needed_symbols,
                        );
                        for parameter in &method_declaration.parameters {
                            collect_type_names(
                                &parameter.param_type,
                                source_text,
                                &mut needed_symbols,
                            );
                        }
                        if let Some(body) = &method_declaration.body {
                            let slice = get_span_slice(body.span, source_text);
                            collect_identifiers_from_slice(slice, &mut needed_symbols);
                        }
                    }
                }
                Declaration::Field(field_declaration) => {
                    let field_name = get_span_slice(field_declaration.name_span, source_text);
                    let namespace_segments: Vec<&str> = field_declaration
                        .namespace_path
                        .iter()
                        .map(|span| get_span_slice(*span, source_text))
                        .filter(|segment| !segment.is_empty())
                        .collect();
                    let namespace = namespace_segments.join("::");
                    let qualified_name = if namespace.is_empty() {
                        field_name.to_string()
                    } else {
                        format!("{namespace}::{field_name}")
                    };
                    if needed_symbols.contains(field_name)
                        || needed_symbols.contains(&qualified_name)
                        || imported_namespaces.contains(&namespace)
                        || namespace.starts_with(&core_namespace_prefix)
                    {
                        collect_type_names(
                            &field_declaration.field_type,
                            source_text,
                            &mut needed_symbols,
                        );
                        if field_declaration.initializer.is_some() {
                            let slice = get_span_slice(field_declaration.span, source_text);
                            collect_identifiers_from_slice(slice, &mut needed_symbols);
                        }
                    }
                }
                _ => {}
            }
        }

        if needed_symbols.len() == size_before {
            break;
        }
    }

    // Retained for future explicit namespace-level whitelisting if needed.
    let _ = &imported_namespaces;

    ast.declarations.retain(|declaration| match declaration {
        Declaration::Class(class_declaration) => {
            let class_name = get_span_slice(class_declaration.name_span, source_text);
            let namespace_segments: Vec<&str> = class_declaration
                .namespace_path
                .iter()
                .map(|span| get_span_slice(*span, source_text))
                .filter(|segment| !segment.is_empty())
                .collect();
            let namespace = namespace_segments.join("::");
            let qualified_name = if namespace.is_empty() {
                class_name.to_string()
            } else {
                format!("{namespace}::{class_name}")
            };

            needed_symbols.contains(class_name)
                || needed_symbols.contains(&qualified_name)
                || imported_namespaces.contains(&namespace)
                || namespace.starts_with(&format!("{package_name}::core"))
        }
        Declaration::Interface(interface_declaration) => {
            let iface_name = get_span_slice(interface_declaration.name_span, source_text);
            let namespace_segments: Vec<&str> = interface_declaration
                .namespace_path
                .iter()
                .map(|span| get_span_slice(*span, source_text))
                .filter(|segment| !segment.is_empty())
                .collect();
            let namespace = namespace_segments.join("::");
            let qualified_name = if namespace.is_empty() {
                iface_name.to_string()
            } else {
                format!("{namespace}::{iface_name}")
            };

            needed_symbols.contains(iface_name)
                || needed_symbols.contains(&qualified_name)
                || imported_namespaces.contains(&namespace)
                || namespace.starts_with(&format!("{package_name}::core"))
        }
        Declaration::Enum(enum_declaration) => {
            let enum_name = get_span_slice(enum_declaration.name_span, source_text);
            let namespace_segments: Vec<&str> = enum_declaration
                .namespace_path
                .iter()
                .map(|span| get_span_slice(*span, source_text))
                .filter(|segment| !segment.is_empty())
                .collect();
            let namespace = namespace_segments.join("::");
            let qualified_name = if namespace.is_empty() {
                enum_name.to_string()
            } else {
                format!("{namespace}::{enum_name}")
            };

            needed_symbols.contains(enum_name)
                || needed_symbols.contains(&qualified_name)
                || imported_namespaces.contains(&namespace)
                || namespace.starts_with(&format!("{package_name}::core"))
        }
        Declaration::Method(method_declaration) => {
            let method_name = get_span_slice(method_declaration.name_span, source_text);
            let namespace_segments: Vec<&str> = method_declaration
                .namespace_path
                .iter()
                .map(|span| get_span_slice(*span, source_text))
                .filter(|segment| !segment.is_empty())
                .collect();
            let namespace = namespace_segments.join("::");
            let qualified_name = if namespace.is_empty() {
                method_name.to_string()
            } else {
                format!("{namespace}::{method_name}")
            };

            needed_symbols.contains(method_name)
                || needed_symbols.contains(&qualified_name)
                || imported_namespaces.contains(&namespace)
                || namespace.starts_with(&format!("{package_name}::core"))
        }
        Declaration::Field(field_declaration) => {
            let field_name = get_span_slice(field_declaration.name_span, source_text);
            let namespace_segments: Vec<&str> = field_declaration
                .namespace_path
                .iter()
                .map(|span| get_span_slice(*span, source_text))
                .filter(|segment| !segment.is_empty())
                .collect();
            let namespace = namespace_segments.join("::");
            let qualified_name = if namespace.is_empty() {
                field_name.to_string()
            } else {
                format!("{namespace}::{field_name}")
            };

            needed_symbols.contains(field_name)
                || needed_symbols.contains(&qualified_name)
                || imported_namespaces.contains(&namespace)
                || namespace.starts_with(&format!("{package_name}::core"))
        }
        _ => true,
    });
}
