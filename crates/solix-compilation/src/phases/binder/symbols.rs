//! Semantic symbol table, lexical scoping, and symbol resolution infrastructure.

#![forbid(unsafe_code)]

use crate::phases::lexing::Span;
use crate::phases::parser::statements::Modifier;
use serde::{Deserialize, Serialize};
use solix_common::Type;
use std::collections::HashMap;

/// Unique sequential identifier indexing a symbol in the compiler's symbol table.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct SymbolId(pub u32);

/// Categorical kind of a declared symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Class type declaration.
    Class,
    /// Interface type declaration.
    Interface,
    /// Enumeration type declaration.
    Enum,
    /// Individual enum variant value.
    EnumVariant,
    /// Class field or global variable.
    Field,
    /// Function or class method.
    Method,
    /// Class constructor.
    Constructor,
    /// Class destructor.
    Destructor,
    /// Operator overload.
    Operator,
    /// Formal parameter of a method, function, or constructor.
    Parameter,
    /// Local variable in a block scope.
    LocalVariable,
    /// Type alias statement.
    Alias,
}

/// Metadata record associated with a registered symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEntry {
    /// Unique identifier assigned to this symbol.
    pub id: SymbolId,
    /// Short unqualified identifier name (e.g., `"List"`).
    pub name: String,
    /// Fully qualified namespace name (e.g., `"std.collections.List"`).
    pub fully_qualified_name: String,
    /// Source file identifier where this symbol was declared.
    pub source_id: u16,
    /// Source coordinate span of the symbol's declaration.
    pub span: Span,
    /// Declaration modifiers (`public`, `private`, `static`, etc.).
    pub modifiers: Vec<Modifier>,
    /// Associated evaluated semantic type.
    pub symbol_type: Type,
    /// Optional enclosing parent symbol ID (e.g., enclosing class for a method).
    pub parent_symbol: Option<SymbolId>,
    /// Child member symbols (fields, methods, constructors) if this is a composite type.
    pub members: Vec<SymbolId>,
    /// Whether this symbol is declared with the `static` modifier.
    pub is_static: bool,
    /// Whether this symbol represents an immutable constant (`const`).
    pub is_const: bool,
    /// Local frame stack slot index (for parameters and local variables).
    pub slot_index: Option<u32>,
}

impl SymbolEntry {
    /// Returns whether this symbol is publicly accessible outside its declaring package.
    #[must_use]
    pub fn is_public(&self) -> bool {
        self.modifiers.contains(&Modifier::Public)
    }

    /// Returns whether this symbol is private to its declaring type.
    #[must_use]
    pub fn is_private(&self) -> bool {
        self.modifiers.contains(&Modifier::Private)
    }

    /// Returns whether this symbol is protected (accessible to subclasses).
    #[must_use]
    pub fn is_protected(&self) -> bool {
        self.modifiers.contains(&Modifier::Protected)
    }

    /// Formats the signature of a function/method symbol in canonical Solix signature notation:
    /// `<return_type> <name>(<param_type_1>, <param_type_2>, ...)`
    ///
    /// Omits access modifiers and variable names. Uses `::` for namespace qualification.
    /// Example: `solix::models::User find_user_by_email(int64, std::string, bool)`
    #[must_use]
    pub fn to_signature_string(&self) -> String {
        if let Type::FunctionPointer {
            return_type,
            parameter_types,
        } = &self.symbol_type
        {
            let params = parameter_types
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            format!("{return_type} {}({params})", self.name)
        } else {
            format!("{} {}", self.symbol_type, self.name)
        }
    }
}

/// Representation of an import statement in a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEntry {
    /// Source file index where the import was declared.
    pub source_id: u16,
    /// The imported path or package string (e.g., `"std.collections.List"` or `"std.math"`).
    pub path: String,
    /// Short symbol name (e.g., `"List"` or `""` for wildcard imports).
    pub symbol_name: String,
    /// Optional imported alias identifier (`import std.io.File as SolixFile`).
    pub alias: Option<String>,
    /// Whether this import is a wildcard package import (`import std.math.*;`).
    pub is_wildcard: bool,
    /// Coordinate span of the import statement.
    pub span: Span,
}

/// Hierarchical compiler symbol table managing global, per-file, and local lexical scopes.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// All registered symbol entries indexed by [`SymbolId`].
    pub symbols: Vec<SymbolEntry>,
    /// Mapping from fully qualified symbol names to their [`SymbolId`].
    pub global_names: HashMap<String, SymbolId>,
    /// Per-source public symbol exports: `source_exports[source_id]` maps short name to [`SymbolId`].
    pub source_exports: Vec<HashMap<String, SymbolId>>,
    /// Declared package prefix per source file (e.g. `"std.collections"`).
    pub file_packages: Vec<String>,
    /// Imported symbols and wildcards per source file.
    pub file_imports: Vec<Vec<ImportEntry>>,
    /// Active lexical scope stack for local variables and parameters.
    pub scope_stack: Vec<HashMap<String, SymbolId>>,
}

impl SymbolTable {
    /// Creates a new empty symbol table initialized for the specified number of source files.
    #[must_use]
    pub fn with_sources_count(sources_count: usize) -> Self {
        let mut source_exports = Vec::with_capacity(sources_count);
        let mut file_packages = Vec::with_capacity(sources_count);
        let mut file_imports = Vec::with_capacity(sources_count);

        for _ in 0..sources_count {
            source_exports.push(HashMap::new());
            file_packages.push(String::new());
            file_imports.push(Vec::new());
        }

        Self {
            symbols: Vec::new(),
            global_names: HashMap::new(),
            source_exports,
            file_packages,
            file_imports,
            scope_stack: Vec::new(),
        }
    }

    /// Registers a new symbol in the symbol table, assigning a sequential [`SymbolId`].
    pub fn insert_symbol(&mut self, mut entry: SymbolEntry) -> SymbolId {
        let assigned_id = SymbolId(self.symbols.len() as u32);
        entry.id = assigned_id;

        if !entry.fully_qualified_name.is_empty() {
            self.global_names
                .insert(entry.fully_qualified_name.clone(), assigned_id);
        }

        let source_index = entry.source_id as usize;
        if source_index < self.source_exports.len() && entry.is_public() {
            self.source_exports[source_index].insert(entry.name.clone(), assigned_id);
        }

        self.symbols.push(entry);
        assigned_id
    }

    /// Retrieves an immutable reference to a symbol by its [`SymbolId`].
    #[must_use]
    pub fn get_symbol(&self, symbol_id: SymbolId) -> Option<&SymbolEntry> {
        self.symbols.get(symbol_id.0 as usize)
    }

    /// Retrieves a mutable reference to a symbol by its [`SymbolId`].
    pub fn get_symbol_mut(&mut self, symbol_id: SymbolId) -> Option<&mut SymbolEntry> {
        self.symbols.get_mut(symbol_id.0 as usize)
    }

    /// Looks up a symbol by its exact fully qualified name or dot-normalized name.
    #[must_use]
    pub fn lookup_global(&self, fully_qualified_name: &str) -> Option<SymbolId> {
        if let Some(&symbol_id) = self.global_names.get(fully_qualified_name) {
            return Some(symbol_id);
        }
        let normalized = fully_qualified_name.replace('.', "::");
        self.global_names.get(&normalized).copied()
    }

    /// Pushes a new lexical block scope onto the active scope stack.
    pub fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Pops the current lexical block scope from the scope stack.
    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Declares a local variable or parameter in the innermost lexical scope.
    pub fn declare_local(&mut self, name: impl Into<String>, symbol_id: SymbolId) {
        if let Some(innermost_scope) = self.scope_stack.last_mut() {
            innermost_scope.insert(name.into(), symbol_id);
        }
    }

    /// Resolves an unqualified or qualified symbol identifier from the perspective of `source_id`.
    ///
    /// Resolution precedence (SPEC-0001 §11):
    /// 1. Lexical local scopes (innermost to outermost).
    /// 2. Explicit imports in `source_id`.
    /// 3. Package-local declarations (`package_prefix::name`).
    /// 4. Wildcard imports in `source_id`.
    /// 5. Global fully qualified name direct match.
    #[must_use]
    pub fn resolve_symbol(&self, source_id: u16, identifier: &str) -> Option<SymbolId> {
        // 1. Check local scopes
        for scope in self.scope_stack.iter().rev() {
            if let Some(&symbol_id) = scope.get(identifier) {
                return Some(symbol_id);
            }
        }

        let source_index = source_id as usize;
        let normalized_identifier = identifier.replace('.', "::");

        // 2. Check explicit imports
        if let Some(imports) = self.file_imports.get(source_index) {
            for import_entry in imports {
                if !import_entry.is_wildcard {
                    if let Some(alias) = &import_entry.alias {
                        if alias == identifier || alias == &normalized_identifier {
                            return self.lookup_global(&import_entry.path);
                        }
                    } else if import_entry.symbol_name == identifier
                        || import_entry.symbol_name == normalized_identifier
                    {
                        return self.lookup_global(&import_entry.path);
                    }
                }
            }
        }

        // 3. Check current package
        if let Some(package_prefix) = self.file_packages.get(source_index)
            && !package_prefix.is_empty()
        {
            let package_qualified = format!("{package_prefix}::{identifier}");
            if let Some(symbol_id) = self.lookup_global(&package_qualified) {
                return Some(symbol_id);
            }
        }

        // 4. Check wildcard imports
        if let Some(imports) = self.file_imports.get(source_index) {
            for import_entry in imports {
                if import_entry.is_wildcard {
                    let candidate_path = format!("{}::{}", import_entry.path, identifier);
                    if let Some(symbol_id) = self.lookup_global(&candidate_path) {
                        return Some(symbol_id);
                    }
                }
            }
        }

        // 5. Global FQN direct match
        if let Some(symbol_id) = self.lookup_global(identifier) {
            return Some(symbol_id);
        }

        // 6. Unqualified unique package fallback (root facade for root/default package)
        if !identifier.contains("::")
            && !identifier.contains('.')
            && self
                .file_packages
                .get(source_index)
                .is_none_or(|package_prefix| package_prefix.is_empty())
        {
            let suffix = format!("::{identifier}");
            let mut matched_symbol_id = None;
            for (candidate_fqn, &candidate_symbol_id) in &self.global_names {
                if candidate_fqn.ends_with(&suffix) {
                    if matched_symbol_id.is_some() {
                        // Ambiguous across multiple packages
                        return None;
                    }
                    matched_symbol_id = Some(candidate_symbol_id);
                }
            }
            if let Some(symbol_id) = matched_symbol_id {
                return Some(symbol_id);
            }
        }

        None
    }

    /// Finds all candidate fully qualified symbol paths matching an identifier across active wildcard imports in `source_id`.
    #[must_use]
    pub fn find_wildcard_matches(&self, source_id: u16, identifier: &str) -> Vec<String> {
        let source_index = source_id as usize;
        let mut matches = Vec::new();
        if let Some(imports) = self.file_imports.get(source_index) {
            for import_entry in imports {
                if import_entry.is_wildcard {
                    let candidate_path = format!("{}::{}", import_entry.path, identifier);
                    if self.lookup_global(&candidate_path).is_some() {
                        matches.push(candidate_path);
                    }
                }
            }
        }
        matches
    }
}
