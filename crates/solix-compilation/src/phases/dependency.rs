//! Phase 0: Dependency resolution and synthetic API extraction phase.
//!
//! Executes before lexical analysis to resolve all explicit (`--deps`) and pre-loaded
//! dependencies, verify Semantic Versioning compatibility across the entire dependency graph,
//! and synthesize API interfaces (`manifest_to_source`) directly into the compilation context.

#![forbid(unsafe_code)]

use crate::CompilationContext::CompilationContext;
use crate::manifest::manifest_to_source;
use crate::phases::lexing::Span;
use crate::phases::phase::CompilationPhase;
use miette::SourceSpan;
use solix_common::CompilationDiagnostic;
use solix_common::bytecode::BinaryProgram;
use solix_common::diagnostics::binding::BindingError;
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
    let mut solved_map: HashMap<String, (PackageVersion, BinaryProgram)> = HashMap::new();
    let mut accumulated_diagnostics = Vec::new();

    // 1. Enqueue pre-loaded binary programs
    for binary_program in &context.options.dependency_binaries {
        if !binary_program.package_name.is_empty() {
            let parsed_version = PackageVersion::parse(&binary_program.package_version)
                .unwrap_or_else(|_| PackageVersion::new(0, 1, 0));
            solved_map.insert(
                binary_program.package_name.clone(),
                (parsed_version, binary_program.clone()),
            );
            for transitive_requirement in &binary_program.export_manifest.dependencies {
                work_queue.push_back((
                    transitive_requirement.clone(),
                    binary_program.package_name.clone(),
                ));
            }
        }
    }

    // 2. Enqueue command-line explicit dependencies
    for dependency_spec in &context.options.dependency_specs {
        work_queue.push_back((dependency_spec.clone(), "compiler options".to_string()));
    }

    // 3. Resolve the transitive requirement queue
    while let Some((requirement, requester_name)) = work_queue.pop_front() {
        if let Some((existing_version, existing_program)) = solved_map.get_mut(&requirement.name) {
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

            if *existing_version < requirement.version {
                if let Some(solver) = &context.solver {
                    match solver.resolve_compatible(&requirement.name, requirement.version) {
                        Ok(upgraded_package) => {
                            *existing_version = upgraded_package.version;
                            *existing_program = upgraded_package.program.clone();
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
            match solver.resolve_compatible(&requirement.name, requirement.version) {
                Ok(solved_package) => {
                    let program = solved_package.program.clone();
                    let version = solved_package.version;
                    solved_map.insert(requirement.name.clone(), (version, program.clone()));
                    for transitive in &program.export_manifest.dependencies {
                        work_queue.push_back((transitive.clone(), requirement.name.clone()));
                    }
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
    let mut sorted_packages: Vec<(String, PackageVersion, BinaryProgram)> = solved_map
        .into_iter()
        .map(|(package_name, (version, program))| (package_name, version, program))
        .collect();
    sorted_packages.sort_by(|package_a, package_b| package_a.0.cmp(&package_b.0));

    for (package_name, version, program) in sorted_packages {
        context.dependencies_map.insert(
            package_name.clone(),
            DependencyRequirement {
                name: package_name.clone(),
                version,
            },
        );

        if context.loaded_api_packages.insert(package_name.clone()) {
            let manifest_json =
                serde_json::to_string(&program.export_manifest).map_err(|serialize_error| {
                    vec![CompilationDiagnostic::from(
                        BindingError::ExternalPackageNotFound {
                            name: package_name.clone(),
                            reason: format!(
                                "Failed to serialize export manifest: {serialize_error}"
                            ),
                            span: SourceSpan::from(0..0),
                        },
                    )]
                })?;

            let (source_name, source_code) =
                manifest_to_source(&manifest_json).map_err(|synthesis_error| {
                    vec![CompilationDiagnostic::from(
                        BindingError::ExternalPackageNotFound {
                            name: package_name.clone(),
                            reason: format!("Failed to synthesize API source: {synthesis_error}"),
                            span: SourceSpan::from(0..0),
                        },
                    )]
                })?;

            context.add_source(source_name, source_code);
        }
    }

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
    context.dependencies_map.insert(
        package_name.to_string(),
        DependencyRequirement {
            name: package_name.to_string(),
            version: solved_package.version,
        },
    );

    // Synthesize API, lex, and parse into context
    if context.loaded_api_packages.insert(package_name.to_string()) {
        let manifest_json = serde_json::to_string(&solved_package.program.export_manifest)
            .map_err(|serialize_error| {
                vec![CompilationDiagnostic::from(
                    BindingError::ExternalPackageNotFound {
                        name: package_name.to_string(),
                        reason: format!("Failed to serialize export manifest: {serialize_error}"),
                        span: import_span.to_source_span(),
                    },
                )]
            })?;

        let (source_name, source_code) =
            manifest_to_source(&manifest_json).map_err(|synthesis_error| {
                vec![CompilationDiagnostic::from(
                    BindingError::ExternalPackageNotFound {
                        name: package_name.to_string(),
                        reason: format!("Failed to synthesize API source: {synthesis_error}"),
                        span: import_span.to_source_span(),
                    },
                )]
            })?;

        let source_id = context.add_source(source_name, &source_code);

        let (tokens, lex_errors) =
            crate::phases::lexing::lexer::tokenize_source(source_id, &source_code);
        if !lex_errors.is_empty() {
            let diagnostics = lex_errors
                .into_iter()
                .map(|err| CompilationDiagnostic::from(err).with_source_id(source_id))
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
                .map(|err| CompilationDiagnostic::from(err).with_source_id(source_id))
                .collect::<Vec<_>>()
        })?;

        context.ast[source_id as usize] = Some(ast);
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

    context.dependencies_map.insert(
        requirement.name.clone(),
        DependencyRequirement {
            name: requirement.name.clone(),
            version: solved_package.version,
        },
    );

    if context.loaded_api_packages.insert(requirement.name.clone()) {
        let manifest_json = serde_json::to_string(&solved_package.program.export_manifest)
            .map_err(|serialize_error| {
                vec![CompilationDiagnostic::from(
                    BindingError::ExternalPackageNotFound {
                        name: requirement.name.clone(),
                        reason: format!("Failed to serialize export manifest: {serialize_error}"),
                        span: import_span.to_source_span(),
                    },
                )]
            })?;

        let (source_name, source_code) =
            manifest_to_source(&manifest_json).map_err(|synthesis_error| {
                vec![CompilationDiagnostic::from(
                    BindingError::ExternalPackageNotFound {
                        name: requirement.name.clone(),
                        reason: format!("Failed to synthesize API source: {synthesis_error}"),
                        span: import_span.to_source_span(),
                    },
                )]
            })?;

        let source_id = context.add_source(source_name, &source_code);

        let (tokens, lex_errors) =
            crate::phases::lexing::lexer::tokenize_source(source_id, &source_code);
        if !lex_errors.is_empty() {
            let diagnostics = lex_errors
                .into_iter()
                .map(|err| CompilationDiagnostic::from(err).with_source_id(source_id))
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
                .map(|err| CompilationDiagnostic::from(err).with_source_id(source_id))
                .collect::<Vec<_>>()
        })?;

        context.ast[source_id as usize] = Some(ast);
    }

    Ok(())
}
