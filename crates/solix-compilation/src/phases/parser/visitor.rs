//! Extensible Visitor pattern for traversing and analyzing Solix Abstract Syntax Trees.

#![forbid(unsafe_code)]

use crate::phases::parser::statements::*;

/// Trait defining the Visitor pattern over Solix AST structures.
///
/// Provides default recursive walking implementations for all nodes, allowing
/// implementors (type checkers, bytecode compilers, linters, formatters) to only
/// override the specific node handlers they require.
pub trait AstVisitor<R = ()> {
    /// Visits the root compilation unit.
    fn visit_program(&mut self, program: &ProgramAst) -> R
    where
        R: Default,
    {
        walk_program(self, program);
        R::default()
    }

    /// Visits a package declaration statement.
    fn visit_package_statement(&mut self, _node: &PackageStatement) -> R
    where
        R: Default,
    {
        R::default()
    }

    /// Visits an import statement.
    fn visit_import_statement(&mut self, _node: &ImportStatement) -> R
    where
        R: Default,
    {
        R::default()
    }

    /// Visits a type alias statement.
    fn visit_alias_statement(&mut self, node: &AliasStatement) -> R
    where
        R: Default,
    {
        self.visit_type_annotation(&node.target_type);
        R::default()
    }

    /// Visits a top-level or member declaration.
    fn visit_declaration(&mut self, node: &Declaration) -> R
    where
        R: Default,
    {
        walk_declaration(self, node);
        R::default()
    }

    /// Visits a class declaration.
    fn visit_class_declaration(&mut self, node: &ClassDeclaration) -> R
    where
        R: Default,
    {
        walk_class_declaration(self, node);
        R::default()
    }

    /// Visits an interface declaration.
    fn visit_interface_declaration(&mut self, node: &InterfaceDeclaration) -> R
    where
        R: Default,
    {
        walk_interface_declaration(self, node);
        R::default()
    }

    /// Visits an enumeration declaration.
    fn visit_enum_declaration(&mut self, node: &EnumDeclaration) -> R
    where
        R: Default,
    {
        walk_enum_declaration(self, node);
        R::default()
    }

    /// Visits a field declaration.
    fn visit_field_declaration(&mut self, node: &FieldDeclaration) -> R
    where
        R: Default,
    {
        walk_field_declaration(self, node);
        R::default()
    }

    /// Visits a method declaration.
    fn visit_method_declaration(&mut self, node: &MethodDeclaration) -> R
    where
        R: Default,
    {
        walk_method_declaration(self, node);
        R::default()
    }

    /// Visits a constructor declaration.
    fn visit_constructor_declaration(&mut self, node: &ConstructorDeclaration) -> R
    where
        R: Default,
    {
        walk_constructor_declaration(self, node);
        R::default()
    }

    /// Visits a destructor declaration.
    fn visit_destructor_declaration(&mut self, node: &DestructorDeclaration) -> R
    where
        R: Default,
    {
        self.visit_block_statement(&node.body);
        R::default()
    }

    /// Visits an operator overload declaration.
    fn visit_operator_declaration(&mut self, node: &OperatorDeclaration) -> R
    where
        R: Default,
    {
        walk_operator_declaration(self, node);
        R::default()
    }

    /// Visits a statement.
    fn visit_statement(&mut self, node: &Statement) -> R
    where
        R: Default,
    {
        walk_statement(self, node);
        R::default()
    }

    /// Visits a compound block statement.
    fn visit_block_statement(&mut self, node: &BlockStatement) -> R
    where
        R: Default,
    {
        for statement in &node.statements {
            self.visit_statement(statement);
        }
        R::default()
    }

    /// Visits an if-conditional branch statement.
    fn visit_if_statement(&mut self, node: &IfStatement) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.condition);
        self.visit_statement(&node.then_branch);
        if let Some(else_branch) = &node.else_branch {
            self.visit_statement(else_branch);
        }
        R::default()
    }

    /// Visits a while loop statement.
    fn visit_while_statement(&mut self, node: &WhileStatement) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.condition);
        self.visit_statement(&node.body);
        R::default()
    }

    /// Visits a do-while loop statement.
    fn visit_do_while_statement(&mut self, node: &DoWhileStatement) -> R
    where
        R: Default,
    {
        self.visit_statement(&node.body);
        self.visit_expression(&node.condition);
        R::default()
    }

    /// Visits a 3-clause for loop statement.
    fn visit_for_statement(&mut self, node: &ForStatement) -> R
    where
        R: Default,
    {
        if let Some(initializer) = &node.initializer {
            self.visit_statement(initializer);
        }
        if let Some(condition) = &node.condition {
            self.visit_expression(condition);
        }
        if let Some(update) = &node.update {
            self.visit_expression(update);
        }
        self.visit_statement(&node.body);
        R::default()
    }

    /// Visits a switch-case statement.
    fn visit_switch_statement(&mut self, node: &SwitchStatement) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.discriminant);
        for case_clause in &node.cases {
            self.visit_case_clause(case_clause);
        }
        if let Some(default_stmts) = &node.default_case {
            for statement in default_stmts {
                self.visit_statement(statement);
            }
        }
        R::default()
    }

    /// Visits a single case clause in a switch statement.
    fn visit_case_clause(&mut self, node: &CaseClause) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.case_value);
        for statement in &node.statements {
            self.visit_statement(statement);
        }
        R::default()
    }

    /// Visits an exception handling try statement.
    fn visit_try_statement(&mut self, node: &TryStatement) -> R
    where
        R: Default,
    {
        self.visit_block_statement(&node.try_block);
        for catch_clause in &node.catch_clauses {
            self.visit_catch_clause(catch_clause);
        }
        if let Some(finally_block) = &node.finally_block {
            self.visit_block_statement(finally_block);
        }
        R::default()
    }

    /// Visits a catch clause.
    fn visit_catch_clause(&mut self, node: &CatchClause) -> R
    where
        R: Default,
    {
        self.visit_type_annotation(&node.exception_type);
        self.visit_block_statement(&node.block);
        R::default()
    }

    /// Visits an exception throw statement.
    fn visit_throw_statement(&mut self, node: &ThrowStatement) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.expression);
        R::default()
    }

    /// Visits a function/method return statement.
    fn visit_return_statement(&mut self, node: &ReturnStatement) -> R
    where
        R: Default,
    {
        if let Some(value) = &node.value {
            self.visit_expression(value);
        }
        R::default()
    }

    /// Visits a loop break statement.
    fn visit_break_statement(&mut self, _node: &BreakStatement) -> R
    where
        R: Default,
    {
        R::default()
    }

    /// Visits a loop continue statement.
    fn visit_continue_statement(&mut self, _node: &ContinueStatement) -> R
    where
        R: Default,
    {
        R::default()
    }

    /// Visits a variable declaration statement.
    fn visit_variable_declaration(&mut self, node: &VariableDeclarationStatement) -> R
    where
        R: Default,
    {
        self.visit_type_annotation(&node.var_type);
        if let Some(initializer) = &node.initializer {
            self.visit_expression(initializer);
        }
        R::default()
    }

    /// Visits a bare expression statement.
    fn visit_expression_statement(&mut self, node: &ExpressionStatement) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.expression);
        R::default()
    }

    /// Visits an assertion intrinsic statement.
    fn visit_assert_statement(&mut self, node: &AssertStatement) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.condition);
        if let Some(message) = &node.message {
            self.visit_expression(message);
        }
        R::default()
    }

    /// Visits a process exit statement.
    fn visit_exit_statement(&mut self, node: &ExitStatement) -> R
    where
        R: Default,
    {
        self.visit_expression(&node.code);
        R::default()
    }

    /// Visits an expression node.
    fn visit_expression(&mut self, node: &Expression) -> R
    where
        R: Default,
    {
        walk_expression(self, node);
        R::default()
    }

    /// Visits a type annotation node.
    fn visit_type_annotation(&mut self, node: &TypeAnnotation) -> R
    where
        R: Default,
    {
        walk_type_annotation(self, node);
        R::default()
    }
}

/// Recursively walks a ProgramAst node.
pub fn walk_program<V: AstVisitor<R> + ?Sized, R: Default>(visitor: &mut V, program: &ProgramAst) {
    if let Some(package) = &program.package {
        visitor.visit_package_statement(package);
    }
    for import_stmt in &program.imports {
        visitor.visit_import_statement(import_stmt);
    }
    for alias_stmt in &program.aliases {
        visitor.visit_alias_statement(alias_stmt);
    }
    for declaration in &program.declarations {
        visitor.visit_declaration(declaration);
    }
    for statement in &program.root_statements {
        visitor.visit_statement(statement);
    }
}

/// Recursively walks a Declaration node.
pub fn walk_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &Declaration,
) {
    match node {
        Declaration::Class(decl) => {
            visitor.visit_class_declaration(decl);
        }
        Declaration::Interface(decl) => {
            visitor.visit_interface_declaration(decl);
        }
        Declaration::Enum(decl) => {
            visitor.visit_enum_declaration(decl);
        }
        Declaration::Field(decl) => {
            visitor.visit_field_declaration(decl);
        }
        Declaration::Method(decl) => {
            visitor.visit_method_declaration(decl);
        }
        Declaration::Constructor(decl) => {
            visitor.visit_constructor_declaration(decl);
        }
        Declaration::Destructor(decl) => {
            visitor.visit_destructor_declaration(decl);
        }
        Declaration::Operator(decl) => {
            visitor.visit_operator_declaration(decl);
        }
    }
}

/// Recursively walks a ClassDeclaration node.
pub fn walk_class_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &ClassDeclaration,
) {
    if let Some(super_class) = &node.super_class {
        visitor.visit_type_annotation(super_class);
    }
    for interface in &node.interfaces {
        visitor.visit_type_annotation(interface);
    }
    for member in &node.members {
        match member {
            ClassMember::Field(decl) => {
                visitor.visit_field_declaration(decl);
            }
            ClassMember::Method(decl) => {
                visitor.visit_method_declaration(decl);
            }
            ClassMember::Constructor(decl) => {
                visitor.visit_constructor_declaration(decl);
            }
            ClassMember::Destructor(decl) => {
                visitor.visit_destructor_declaration(decl);
            }
            ClassMember::Operator(decl) => {
                visitor.visit_operator_declaration(decl);
            }
            ClassMember::Enum(decl) => {
                visitor.visit_enum_declaration(decl);
            }
            ClassMember::Class(decl) => {
                visitor.visit_class_declaration(decl);
            }
            ClassMember::Interface(decl) => {
                visitor.visit_interface_declaration(decl);
            }
        }
    }
}

/// Recursively walks an InterfaceDeclaration node.
pub fn walk_interface_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &InterfaceDeclaration,
) {
    for interface in &node.super_interfaces {
        visitor.visit_type_annotation(interface);
    }
    for method in &node.methods {
        visitor.visit_method_declaration(method);
    }
}

/// Recursively walks an EnumDeclaration node.
pub fn walk_enum_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &EnumDeclaration,
) {
    for variant in &node.variants {
        if let Some(explicit_val) = &variant.explicit_value {
            visitor.visit_expression(explicit_val);
        }
    }
}

/// Recursively walks a FieldDeclaration node.
pub fn walk_field_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &FieldDeclaration,
) {
    visitor.visit_type_annotation(&node.field_type);
    if let Some(initializer) = &node.initializer {
        visitor.visit_expression(initializer);
    }
}

/// Recursively walks a MethodDeclaration node.
pub fn walk_method_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &MethodDeclaration,
) {
    visitor.visit_type_annotation(&node.return_type);
    for param in &node.parameters {
        visitor.visit_type_annotation(&param.param_type);
        if let Some(default_val) = &param.default_value {
            visitor.visit_expression(default_val);
        }
    }
    if let Some(body) = &node.body {
        visitor.visit_block_statement(body);
    }
}

/// Recursively walks a ConstructorDeclaration node.
pub fn walk_constructor_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &ConstructorDeclaration,
) {
    for param in &node.parameters {
        visitor.visit_type_annotation(&param.param_type);
        if let Some(default_val) = &param.default_value {
            visitor.visit_expression(default_val);
        }
    }
    if let Some(super_args) = &node.super_call_args {
        for arg in super_args {
            visitor.visit_expression(arg);
        }
    }
    visitor.visit_block_statement(&node.body);
}

/// Recursively walks an OperatorDeclaration node.
pub fn walk_operator_declaration<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &OperatorDeclaration,
) {
    for param in &node.parameters {
        visitor.visit_type_annotation(&param.param_type);
        if let Some(default_val) = &param.default_value {
            visitor.visit_expression(default_val);
        }
    }
    visitor.visit_type_annotation(&node.return_type);
    visitor.visit_block_statement(&node.body);
}

/// Recursively walks a Statement node.
pub fn walk_statement<V: AstVisitor<R> + ?Sized, R: Default>(visitor: &mut V, node: &Statement) {
    match node {
        Statement::Block(stmt) => {
            visitor.visit_block_statement(stmt);
        }
        Statement::If(stmt) => {
            visitor.visit_if_statement(stmt);
        }
        Statement::While(stmt) => {
            visitor.visit_while_statement(stmt);
        }
        Statement::DoWhile(stmt) => {
            visitor.visit_do_while_statement(stmt);
        }
        Statement::For(stmt) => {
            visitor.visit_for_statement(stmt);
        }
        Statement::Switch(stmt) => {
            visitor.visit_switch_statement(stmt);
        }
        Statement::Try(stmt) => {
            visitor.visit_try_statement(stmt);
        }
        Statement::Throw(stmt) => {
            visitor.visit_throw_statement(stmt);
        }
        Statement::Return(stmt) => {
            visitor.visit_return_statement(stmt);
        }
        Statement::Break(stmt) => {
            visitor.visit_break_statement(stmt);
        }
        Statement::Continue(stmt) => {
            visitor.visit_continue_statement(stmt);
        }
        Statement::VariableDeclaration(stmt) => {
            visitor.visit_variable_declaration(stmt);
        }
        Statement::Expression(stmt) => {
            visitor.visit_expression_statement(stmt);
        }
        Statement::Assert(stmt) => {
            visitor.visit_assert_statement(stmt);
        }
        Statement::Exit(stmt) => {
            visitor.visit_exit_statement(stmt);
        }
        Statement::Declaration(decl) => {
            visitor.visit_declaration(decl);
        }
    }
}

/// Recursively walks an Expression node.
pub fn walk_expression<V: AstVisitor<R> + ?Sized, R: Default>(visitor: &mut V, node: &Expression) {
    match node {
        Expression::Literal { .. } | Expression::Identifier { .. } => {}
        Expression::Binary { left, right, .. } => {
            visitor.visit_expression(left);
            visitor.visit_expression(right);
        }
        Expression::Unary { operand, .. } => {
            visitor.visit_expression(operand);
        }
        Expression::Assignment { target, value, .. } => {
            visitor.visit_expression(target);
            visitor.visit_expression(value);
        }
        Expression::MemberAccess { object, .. } => {
            visitor.visit_expression(object);
        }
        Expression::MethodCall {
            callee, arguments, ..
        } => {
            visitor.visit_expression(callee);
            for arg in arguments {
                visitor.visit_expression(arg);
            }
        }
        Expression::ArrayAccess { array, index, .. } => {
            visitor.visit_expression(array);
            visitor.visit_expression(index);
        }
        Expression::ArrayCreation {
            element_type, size, ..
        } => {
            visitor.visit_type_annotation(element_type);
            visitor.visit_expression(size);
        }
        Expression::ArrayLiteral { elements, .. } => {
            for element in elements {
                visitor.visit_expression(element);
            }
        }
        Expression::NewInstance {
            instance_type,
            arguments,
            ..
        } => {
            visitor.visit_type_annotation(instance_type);
            for arg in arguments {
                visitor.visit_expression(arg);
            }
        }
        Expression::Cast {
            expression,
            target_type,
            ..
        } => {
            visitor.visit_expression(expression);
            visitor.visit_type_annotation(target_type);
        }
        Expression::Instanceof {
            expression,
            target_type,
            ..
        } => {
            visitor.visit_expression(expression);
            visitor.visit_type_annotation(target_type);
        }
        Expression::Ternary {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visitor.visit_expression(condition);
            visitor.visit_expression(then_branch);
            visitor.visit_expression(else_branch);
        }
        Expression::Lambda {
            parameters, body, ..
        } => {
            for param in parameters {
                visitor.visit_type_annotation(&param.param_type);
            }
            match body {
                LambdaBody::Expression(expr) => {
                    visitor.visit_expression(expr);
                }
                LambdaBody::Block(block) => {
                    visitor.visit_block_statement(block);
                }
            }
        }
        Expression::Sizeof { target_type, .. } | Expression::Default { target_type, .. } => {
            visitor.visit_type_annotation(target_type);
        }
        Expression::TupleCreation { elements, .. } => {
            for element in elements {
                visitor.visit_expression(element);
            }
        }
        Expression::Block {
            statements, result, ..
        } => {
            for statement in statements {
                visitor.visit_statement(statement);
            }
            if let Some(res) = result {
                visitor.visit_expression(res);
            }
        }
    }
}

/// Recursively walks a TypeAnnotation node.
pub fn walk_type_annotation<V: AstVisitor<R> + ?Sized, R: Default>(
    visitor: &mut V,
    node: &TypeAnnotation,
) {
    match node {
        TypeAnnotation::Primitive { .. } | TypeAnnotation::Named { .. } => {}
        TypeAnnotation::Generic {
            base, arguments, ..
        } => {
            visitor.visit_type_annotation(base);
            for arg in arguments {
                visitor.visit_type_annotation(arg);
            }
        }
        TypeAnnotation::Array { element_type, .. } => {
            visitor.visit_type_annotation(element_type);
        }
        TypeAnnotation::Tuple { elements, .. } => {
            for element in elements {
                visitor.visit_type_annotation(element);
            }
        }
        TypeAnnotation::FunctionPointer {
            return_type,
            parameters,
            ..
        } => {
            visitor.visit_type_annotation(return_type);
            for param in parameters {
                visitor.visit_type_annotation(param);
            }
        }
    }
}

/// Trait implemented by AST nodes that accept an [`AstVisitor`].
pub trait Visitable {
    /// Accepts a visitor to traverse this AST node.
    fn accept<V: AstVisitor<R>, R: Default>(&self, visitor: &mut V) -> R;
}

impl Visitable for ProgramAst {
    fn accept<V: AstVisitor<R>, R: Default>(&self, visitor: &mut V) -> R {
        visitor.visit_program(self)
    }
}

impl Visitable for Statement {
    fn accept<V: AstVisitor<R>, R: Default>(&self, visitor: &mut V) -> R {
        visitor.visit_statement(self)
    }
}

impl Visitable for Declaration {
    fn accept<V: AstVisitor<R>, R: Default>(&self, visitor: &mut V) -> R {
        visitor.visit_declaration(self)
    }
}

impl Visitable for Expression {
    fn accept<V: AstVisitor<R>, R: Default>(&self, visitor: &mut V) -> R {
        visitor.visit_expression(self)
    }
}
