# Parser Error Codes (`E0101` – `E0200`)

This section documents all syntax diagnostics emitted by [`crates/solix-compilation/src/phases/parser/`](../../../crates/solix-compilation/src/phases/parser) and defined in [`ParsingError`](../../../crates/solix-compilation/src/diagnostics/parsing.rs).

---

### `E0101`: Expected Semicolon
* **Cause:** A statement is missing a mandatory terminating semicolon `;`.
* **Help:** Add a semicolon ';' to terminate this statement.
* **Bad Code:**
  ```solix
  int32 total = 42
  ```
* **Fixed Code:**
  ```solix
  int32 total = 42;
  ```

---

### `E0102`: Expected Identifier
* **Cause:** The parser expected a user-defined name but encountered a keyword, symbol, or delimiter.
* **Help:** Provide a valid symbol identifier name.
* **Bad Code:**
  ```solix
  int32 class = 10;
  ```
* **Fixed Code:**
  ```solix
  int32 class_count = 10;
  ```

---

### `E0103`: Expected Expression
* **Diagnostic Message:** `Expected expression`
* **Variant:** `ParsingError::ExpectedExpression`
* **Resolution:** Provide a valid expression (literal, identifier, calculation, or function call).

---

### `E0104`: Expected Closing Delimiter
* **Cause:** An opening parenthesis `(`, brace `{`, bracket `[`, or angle bracket `<` lacks its matching closing token.
* **Help:** Add the matching closing delimiter (')', '}}', or ']') to balance the expression.
* **Bad Code:**
  ```solix
  if (value > 10 {
      do_something();
  }
  ```
* **Fixed Code:**
  ```solix
  if (value > 10) {
      do_something();
  }
  ```

---

### `E0105`: Expected Type Name
* **Diagnostic Message:** `Expected type name`
* **Variant:** `ParsingError::ExpectedTypeName`
* **Resolution:** Provide a valid primitive or user-defined type name (e.g. int32, string, void).

---

### `E0106`: Invalid Assignment Target
* **Diagnostic Message:** `Invalid assignment target`
* **Variant:** `ParsingError::InvalidAssignmentTarget`
* **Resolution:** The left-hand side of an assignment must be a valid mutable l-value (variable, field, or index).

---

### `E0107`: Invalid Increment Target
* **Diagnostic Message:** `Invalid operand for increment or decrement operator`
* **Variant:** `ParsingError::InvalidIncrementTarget`
* **Resolution:** Increment and decrement operators ('++', '--') can only be applied to mutable variables or fields.

---

### `E0108`: Array Allocation Requires Size
* **Diagnostic Message:** `Array allocation requires size expression`
* **Variant:** `ParsingError::ArrayAllocationRequiresSize`
* **Resolution:** Provide an array length expression between brackets, e.g. 'new int32[10]'.

---

### `E0109`: Expected Lambda Arrow
* **Diagnostic Message:** `Expected '=>' arrow in lambda expression`
* **Variant:** `ParsingError::ExpectedLambdaArrow`
* **Resolution:** Lambda expressions must use '=>' between parameters and body, e.g. '(x) => x + 1'.

---

### `E0110`: Expected Type In Cast
* **Diagnostic Message:** `Expected type name in cast expression`
* **Variant:** `ParsingError::ExpectedTypeInCast`
* **Resolution:** Cast expressions require a destination type: 'cast(expr, TargetType)'.

---

### `E0111`: Package Statement Must Be First
* **Cause:** A `package` statement appeared after import statements or class declarations.
* **Help:** Move the 'package' statement to the top of the file before imports or declarations.
* **Bad Code:**
  ```solix
  import solix.io.Console;
  package my.app;
  ```
* **Fixed Code:**
  ```solix
  package my.app;
  import solix.io.Console;
  ```

---

### `E0112`: Duplicate Package Statement
* **Diagnostic Message:** `Only one 'package' statement is allowed per file`
* **Variant:** `ParsingError::DuplicatePackageStatement`
* **Resolution:** Remove duplicate package statement.

---

### `E0113`: Import Statement After Declaration
* **Cause:** An `import` statement appeared after a class, interface, or function declaration.
* **Help:** Move all 'import' statements to the top of the file before type declarations.
* **Bad Code:**
  ```solix
  public class Application { }
  import solix.io.Console;
  ```
* **Fixed Code:**
  ```solix
  import solix.io.Console;
  public class Application { }
  ```

---

### `E0114`: Class In Local Scope
* **Diagnostic Message:** `Classes cannot be declared inside a function or method body`
* **Variant:** `ParsingError::ClassInLocalScope`
* **Resolution:** Move class declarations to the top level of the compilation unit.

---

### `E0115`: Enum In Local Scope
* **Diagnostic Message:** `Enums cannot be declared inside a function or method body`
* **Variant:** `ParsingError::EnumInLocalScope`
* **Resolution:** Move enum declarations to the top level of the compilation unit.

---

### `E0116`: Interface In Local Scope
* **Diagnostic Message:** `Interfaces cannot be declared inside a function or method body`
* **Variant:** `ParsingError::InterfaceInLocalScope`
* **Resolution:** Move interface declarations to the top level of the compilation unit.

---

### `E0117`: Modifier On Local Variable
* **Diagnostic Message:** `Access modifiers are not allowed on local variables`
* **Variant:** `ParsingError::ModifierOnLocalVariable`
* **Resolution:** Remove 'public', 'private', 'protected', or 'internal' from local variable declaration.

---

### `E0118`: Operator Outside Class
* **Diagnostic Message:** `Operator overloads can only be declared inside a class body`
* **Variant:** `ParsingError::OperatorOutsideClass`
* **Resolution:** Move this operator overload inside a class definition.

---

### `E0119`: Constructor Outside Class
* **Diagnostic Message:** `Constructors can only be declared inside a class body`
* **Variant:** `ParsingError::ConstructorOutsideClass`
* **Resolution:** Move this constructor inside a class definition.

---

### `E0120`: Solitary Declaration In Branch Or Loop
* **Diagnostic Message:** `Variable declarations are not allowed as immediate solitary branch or loop statements`
* **Variant:** `ParsingError::SolitaryDeclarationInBranchOrLoop`
* **Resolution:** Enclose the variable declaration in a block '{{ ... }}'.

---

### `E0121`: Case Label Not Constant
* **Diagnostic Message:** `Case label must be a constant literal`
* **Variant:** `ParsingError::CaseLabelNotConstant`
* **Resolution:** Case labels in switch statements must be integer, character, or string literals.

---

### `E0122`: Statement Before Case In Switch
* **Diagnostic Message:** `Statement without a preceding case in switch`
* **Variant:** `ParsingError::StatementBeforeCaseInSwitch`
* **Resolution:** Move statement inside a 'case' or 'default' branch.

---

### `E0123`: Method Inside Method
* **Diagnostic Message:** `Methods cannot be declared inside another method`
* **Variant:** `ParsingError::MethodInsideMethod`
* **Resolution:** Move method declaration to class or file level, or use a lambda closure instead.

---

### `E0124`: Return Outside Function
* **Diagnostic Message:** `'return' statement outside of function or method body`
* **Variant:** `ParsingError::ReturnOutsideFunction`
* **Resolution:** Return statements can only be executed inside functions, methods, or lambdas.

---

### `E0125`: Interface Destructor
* **Diagnostic Message:** `Interfaces cannot declare destructors`
* **Variant:** `ParsingError::InterfaceDestructor`
* **Resolution:** Remove destructor from interface; interfaces cannot hold state or destructor logic.

---

### `E0126`: Interface Field
* **Diagnostic Message:** `Interfaces must not declare instance fields`
* **Variant:** `ParsingError::InterfaceField`
* **Resolution:** Remove instance fields from interface; interfaces can only define method signatures.

---

### `E0127`: Interface Method With Body
* **Diagnostic Message:** `Interface methods cannot have a body`
* **Variant:** `ParsingError::InterfaceMethodWithBody`
* **Resolution:** Replace the method body with a semicolon ';'.

---

### `E0128`: Statement Directly In Class
* **Diagnostic Message:** `Executable statements or bare blocks are not allowed directly in class body`
* **Variant:** `ParsingError::StatementDirectlyInClass`
* **Resolution:** Move statements inside a method or constructor.

---

### `E0129`: Multiple Destructors In Class
* **Diagnostic Message:** `Class cannot declare multiple destructors`
* **Variant:** `ParsingError::MultipleDestructorsInClass`
* **Resolution:** A class can only have at most one destructor '~ClassName()'.

---

### `E0130`: Invalid Destructor Modifier
* **Cause:** A destructor declaration (`~ClassName()`) was declared with `static`, `virtual`, `override`, or `abstract` modifiers.
* **Help:** Remove invalid modifiers from destructor declaration.
* **Bad Code:**
  ```solix
  public class Connection {
      public virtual ~Connection() { }
  }
  ```
* **Fixed Code:**
  ```solix
  public class Connection {
      ~Connection() { }
  }
  ```

---

### `E0131`: Destructor Name Mismatch
* **Diagnostic Message:** `Destructor name does not match enclosing class name`
* **Variant:** `ParsingError::DestructorNameMismatch`
* **Resolution:** The destructor name must exactly match the enclosing class name prefixed with '~'.

---

### `E0132`: Destructor Parameters
* **Diagnostic Message:** `Destructors cannot accept parameters`
* **Variant:** `ParsingError::DestructorParameters`
* **Resolution:** Remove parameters; destructors must have an empty parameter list '()'.

---

### `E0133`: Constructor Name Mismatch
* **Diagnostic Message:** `Constructor name does not match enclosing class name`
* **Variant:** `ParsingError::ConstructorNameMismatch`
* **Resolution:** The constructor name must exactly match the enclosing class name.

---

### `E0134`: Constructor Return Type
* **Diagnostic Message:** `Constructors must not specify a return type`
* **Variant:** `ParsingError::ConstructorReturnType`
* **Resolution:** Remove return type from constructor declaration.

---

### `E0135`: Invalid Operator Overload
* **Diagnostic Message:** `Operator cannot be overloaded`
* **Variant:** `ParsingError::InvalidOperatorOverload`
* **Resolution:** Only standard arithmetic, comparison, and indexing operators can be overloaded.

---

### `E0136`: Variadic Parameter Not Last
* **Diagnostic Message:** `Variadic parameter must be the last parameter in parameter list`
* **Variant:** `ParsingError::VariadicParameterNotLast`
* **Resolution:** Move the variadic pack parameter to the end of the parameter list.

---

### `E0137`: Extern With Body
* **Diagnostic Message:** `Extern function or method cannot have a body implementation`
* **Variant:** `ParsingError::ExternWithBody`
* **Resolution:** Remove the body '{{ ... }}' and terminate the extern declaration with a semicolon ';'.

---

### `E0138`: Abstract Or Native Method With Body
* **Diagnostic Message:** `Abstract or native method cannot have a body`
* **Variant:** `ParsingError::AbstractOrNativeMethodWithBody`
* **Resolution:** Replace the method body with a semicolon ';'.

---

### `E0139`: Expected Block
* **Diagnostic Message:** `Expected block enclosed in braces '{ ... }'`
* **Variant:** `ParsingError::ExpectedBlock`
* **Resolution:** Provide a block statement enclosed in curly braces '{{ ... }}'.

---

### `E0140`: Try Missing Catch Or Finally
* **Diagnostic Message:** `'try' statement must have at least one 'catch' or 'finally' clause`
* **Variant:** `ParsingError::TryMissingCatchOrFinally`
* **Resolution:** Add a 'catch' block or 'finally' block after the 'try' statement.

---

### `E0141`: Malformed Catch Parameter
* **Diagnostic Message:** `Malformed catch parameter clause`
* **Variant:** `ParsingError::MalformedCatchParameter`
* **Resolution:** Catch clauses must specify an exception type and identifier: 'catch (Exception e)'.

---

### `E0142`: Malformed Switch
* **Diagnostic Message:** `Malformed switch statement condition`
* **Variant:** `ParsingError::MalformedSwitch`
* **Resolution:** Enclose the switch discriminant expression in parentheses: 'switch (expr) {{ ... }}'.

---

### `E0143`: Malformed For Loop
* **Diagnostic Message:** `Malformed for-loop statement`
* **Variant:** `ParsingError::MalformedForLoop`
* **Resolution:** For loops require three semicolon-separated clauses: 'for (init; cond; step)'.

---

### `E0144`: Malformed While Condition
* **Diagnostic Message:** `Malformed while-loop condition`
* **Variant:** `ParsingError::MalformedWhileCondition`
* **Resolution:** Enclose while loop condition in parentheses: 'while (condition)'.

---

### `E0145`: Malformed Do While
* **Diagnostic Message:** `Malformed do-while loop statement`
* **Variant:** `ParsingError::MalformedDoWhile`
* **Resolution:** Do-while loops require 'while (condition);' after the statement body.

---

### `E0146`: Malformed Alias
* **Diagnostic Message:** `Malformed alias statement`
* **Variant:** `ParsingError::MalformedAlias`
* **Resolution:** Alias statements must follow the pattern: 'alias NewName = TargetType;'.

---

### `E0147`: Unexpected Token
* **Diagnostic Message:** `Unexpected token`
* **Variant:** `ParsingError::UnexpectedToken`
* **Resolution:** Remove or replace this unexpected token with valid Solix grammar.

---

### `E0148`: Expression Nesting Depth Exceeded
* **Cause:** An expression exceeds the maximum parser recursion depth ceiling (512 levels of nested sub-expressions or parenthesized clauses).
* **Help:** Simplify the expression by breaking it into intermediate variables.
* **Bad Code:**
  ```solix
  // Deeply nested parenthesized expressions exceeding limit
  ```
* **Fixed Code:**
  ```solix
  int32 intermediate_value = 1;
  int32 result = intermediate_value;
  ```

---

### `E0149`: Required Parameter After Default Parameter
* **Cause:** A required formal parameter follows an optional parameter with a default initializer in a function or method signature.
* **Help:** Move optional parameters with default values after all required parameters.
* **Bad Code:**
  ```solix
  public static int32 compute(int32 base_multiplier = 2, int32 value_operand) {
      return base_multiplier * value_operand;
  }
  ```
* **Fixed Code:**
  ```solix
  public static int32 compute(int32 value_operand, int32 base_multiplier = 2) {
      return value_operand * base_multiplier;
  }
  ```

---

### `E0150`: Integer Literal Overflow
* **Diagnostic Message:** `Integer literal exceeds 64-bit capacity`
* **Variant:** `ParsingError::IntegerLiteralOverflow`
* **Resolution:** Reduce the integer literal value to fit within 64 bits (<= 18446744073709551615).

---

### `E0151`: Invalid Escape Sequence
* **Diagnostic Message:** `Invalid escape sequence in literal`
* **Variant:** `ParsingError::InvalidEscapeSequence`
* **Resolution:** Valid escape sequences are: \n, \t, \r, \\, \', ", \0, \xHH.

---

### `E0152`: Empty Character Literal
* **Diagnostic Message:** `Empty character literal`
* **Variant:** `ParsingError::EmptyCharacterLiteral`
* **Resolution:** Character literals must contain exactly one character: 'a'.

---

### `E0153`: Multi-Character Character Literal
* **Cause:** A character literal contains more than a single character or has extra trailing characters after an escape sequence.
* **Help:** Character literals can only contain one character; use double quotes '
* **Bad Code:**
  ```solix
  char invalid_letter = 'ab';
  char invalid_escape = '\nabc';
  ```
* **Fixed Code:**
  ```solix
  char valid_letter = 'a';
  char valid_escape = '\n';
  ```

---

### `E0154`: Extern With Initializer
* **Diagnostic Message:** `Extern variable declaration cannot have an initializer`
* **Variant:** `ParsingError::ExternWithInitializer`
* **Resolution:** Remove the initializer expression from the extern variable declaration.

---

### `E0155`: Extern Inside Function
* **Diagnostic Message:** `Extern modifier is not allowed inside a local function or block`
* **Variant:** `ParsingError::ExternInsideFunction`
* **Resolution:** Move the extern declaration to package or module scope.

---

### `E0156`: Invalid Extern Modifier
* **Diagnostic Message:** `Extern modifier cannot be combined with '<modifier>'`
* **Variant:** `ParsingError::InvalidExternModifier`
* **Resolution:** Remove either 'extern' or the conflicting modifier.

---

### `E0157`: Dot Not Allowed in Type Path
* **Cause:** Type annotations must separate package/namespace qualifiers with `::` rather than `.`.
* **Help:** Replace '.' with '::' for type and namespace paths (e.g. 'solix::collections::List'). Dot '.' is reserved for instance member access.
* **Bad Code:**
  ```solix
  solix.collections.List<int32> items;
  ```
* **Fixed Code:**
  ```solix
  solix::collections::List<int32> items;
  ```

---

### `E0158`: Unknown Control-Flow Keyword or Statement
* **Cause:** A statement starts with an unrecognized identifier followed by `(...)` and a block `{ ... }` that closely matches a control-flow keyword (`whilje`, `iff`, `foor`, `swich`).
* **Help:** Did you mean '{suggestion}'?
* **Bad Code:**
  ```solix
  public static int32 main() {
      whilje(!WindowShouldClose()) {
          draw_frame();
      }
      return 0;
  }
  ```
* **Fixed Code:**
  ```solix
  public static int32 main() {
      while (!WindowShouldClose()) {
          draw_frame();
      }
      return 0;
  }
  ```

---

### `E0160`: Expected Open Paren
* **Diagnostic Message:** `Expected opening parenthesis '('`
* **Variant:** `ParsingError::ExpectedOpenParen`
* **Resolution:** Add an opening parenthesis '(' to group the expression or argument list.

---

### `E0161`: Expected Close Brace
* **Diagnostic Message:** `Expected closing brace '}'`
* **Variant:** `ParsingError::ExpectedCloseBrace`
* **Resolution:** Add a closing brace '}}' to terminate this block or body.

---

### `E0162`: Expected Open Brace
* **Diagnostic Message:** `Expected opening brace '{'`
* **Variant:** `ParsingError::ExpectedOpenBrace`
* **Resolution:** Add an opening brace '{{' to begin this block or body.

---

### `E0163`: Expected Close Bracket
* **Diagnostic Message:** `Expected closing bracket ']'`
* **Variant:** `ParsingError::ExpectedCloseBracket`
* **Resolution:** Add a closing bracket ']' to terminate the array index or dimension.

---

### `E0164`: Expected Open Bracket
* **Diagnostic Message:** `Expected opening bracket '['`
* **Variant:** `ParsingError::ExpectedOpenBracket`
* **Resolution:** Add an opening bracket '[' to begin the array dimension.

---

### `E0165`: Expected Close Angle
* **Diagnostic Message:** `Expected closing angle bracket '>'`
* **Variant:** `ParsingError::ExpectedCloseAngle`
* **Resolution:** Add a closing angle bracket '>' to close the type argument list.

---

### `E0166`: Expected Colon In Ternary
* **Diagnostic Message:** `Expected ':' in ternary conditional expression`
* **Variant:** `ParsingError::ExpectedColonInTernary`
* **Resolution:** Ternary conditional expressions require a ':' separating the true and false expressions.

---

### `E0167`: Expected Colon In Case
* **Diagnostic Message:** `Expected ':' after case or default label in switch`
* **Variant:** `ParsingError::ExpectedColonInCase`
* **Resolution:** Add a ':' after the case value or default label.

---

### `E0168`: Expected Colon After Super
* **Diagnostic Message:** `Expected ':' before super constructor invocation`
* **Variant:** `ParsingError::ExpectedColonAfterSuper`
* **Resolution:** Add a ':' before 'super(...)' constructor delegation.

---

### `E0169`: Expected Comma In Cast
* **Diagnostic Message:** `Expected ',' between expression and target type in cast expression`
* **Variant:** `ParsingError::ExpectedCommaInCast`
* **Resolution:** Cast expression syntax requires: 'cast(expression, TargetType)'.

---

### `E0170`: Expected Constructor Call Or Size
* **Diagnostic Message:** `Expected constructor call or array size expression after 'new'`
* **Variant:** `ParsingError::ExpectedConstructorCallOrSize`
* **Resolution:** Provide an instantiated type call 'new Type(...)' or array allocation 'new Type[...]'.

---

### `E0171`: Unexpected Question Operator
* **Diagnostic Message:** `Unexpected '?' operator`
* **Variant:** `ParsingError::UnexpectedQuestionOperator`
* **Resolution:** Remove or properly form the ternary conditional expression.

---

### `E0172`: Duplicate Modifier On Declaration
* **Diagnostic Message:** `Duplicate modifier '<modifier>' on declaration`
* **Variant:** `ParsingError::DuplicateModifierOnDeclaration`
* **Resolution:** Remove the redundant modifier.

---

### `E0173`: Conflicting Visibility Modifiers
* **Diagnostic Message:** `Conflicting visibility modifiers: cannot combine '<modifier1>' and '<modifier2>'`
* **Variant:** `ParsingError::ConflictingVisibilityModifiers`
* **Resolution:** Specify at most one visibility modifier ('public', 'private', 'protected', 'internal').

---

### `E0174`: Static On Free Function
* **Diagnostic Message:** `The 'static' modifier is not allowed on package-level functions`
* **Variant:** `ParsingError::StaticOnFreeFunction`
* **Resolution:** Remove 'static'; top-level package functions are already package-scoped.

---

### `E0175`: Standalone Field With Access Modifier
* **Diagnostic Message:** `Access modifier '<modifier>' is not allowed on standalone statements`
* **Variant:** `ParsingError::StandaloneFieldWithAccessModifier`
* **Resolution:** Move variable declarations with access modifiers into a class definition.

---

### `E0176`: Malformed Function Pointer Type
* **Diagnostic Message:** `Malformed function pointer type signature`
* **Variant:** `ParsingError::MalformedFunctionPointerType`
* **Resolution:** Function pointer types must follow: 'ReturnType(*)(ParamType1, ParamType2)'.

---

### `E0177`: Expected Catch Or Finally Block
* **Diagnostic Message:** `Expected 'catch' or 'finally' block after 'try'`
* **Variant:** `ParsingError::ExpectedCatchOrFinallyBlock`
* **Resolution:** A 'try' block must be followed by at least one 'catch' or 'finally' block.

---

### `E0178`: Expected Identifier In Parameter
* **Diagnostic Message:** `Expected identifier in parameter declaration`
* **Variant:** `ParsingError::ExpectedIdentifierInParameter`
* **Resolution:** Provide a parameter name following the parameter type.

---

### `E0179`: Expected Parameter List
* **Diagnostic Message:** `Expected parameter list '(...)' in function or method declaration`
* **Variant:** `ParsingError::ExpectedParameterList`
* **Resolution:** Add parameter list parentheses '()' after the function name.

---

### `E0180`: Expected Interface List
* **Diagnostic Message:** `Expected interface identifier list after 'implements'`
* **Variant:** `ParsingError::ExpectedInterfaceList`
* **Resolution:** Provide one or more comma-separated interface names after 'implements'.

---

### `E0181`: Expected Superclass List
* **Diagnostic Message:** `Expected superclass identifier after 'extends'`
* **Variant:** `ParsingError::ExpectedSuperclassList`
* **Resolution:** Provide the superclass name after 'extends'.

---

### `E0182`: Expected Enum Brace
* **Diagnostic Message:** `Expected '{' enclosing enum variant body`
* **Variant:** `ParsingError::ExpectedEnumBrace`
* **Resolution:** Enclose enum variants in curly braces '{{ ... }}'.

---

### `E0183`: Expected Switch Brace
* **Diagnostic Message:** `Expected '{' enclosing switch body`
* **Variant:** `ParsingError::ExpectedSwitchBrace`
* **Resolution:** Enclose switch cases in curly braces '{{ ... }}'.

---

### `E0184`: Duplicate Default In Switch
* **Diagnostic Message:** `Switch statement cannot contain more than one 'default' clause`
* **Variant:** `ParsingError::DuplicateDefaultInSwitch`
* **Resolution:** Remove the redundant 'default' branch.

---

### `E0185`: Modifier On Module Statement
* **Diagnostic Message:** `Modifiers are not allowed on package or import statements`
* **Variant:** `ParsingError::ModifierOnModuleStatement`
* **Resolution:** Remove modifiers from package or import statements.

---

### `E0186`: Expected Expression In Index
* **Diagnostic Message:** `Expected index expression inside brackets '[]'`
* **Variant:** `ParsingError::ExpectedExpressionInIndex`
* **Resolution:** Provide an expression between index brackets '[index]'.

---

### `E0187`: Empty Tuple Expression
* **Diagnostic Message:** `Empty tuple expressions '()' are not permitted; tuples require at least one element`
* **Variant:** `ParsingError::EmptyTupleExpression`
* **Resolution:** Add at least one element to the tuple: '(item,)' or '(item1, item2)'.

---

### `E0188`: Malformed Type Bound
* **Diagnostic Message:** `Malformed type bound or constraint`
* **Variant:** `ParsingError::MalformedTypeBound`
* **Resolution:** Type bounds must follow valid generic constraint syntax.

---

### `E0189`: Expected Condition In If
* **Diagnostic Message:** `Expected condition expression in 'if' statement`
* **Variant:** `ParsingError::ExpectedConditionInIf`
* **Resolution:** Provide a boolean condition inside parentheses: 'if (condition)'.

---

### `E0190`: Expected Condition In While
* **Diagnostic Message:** `Expected condition expression in 'while' loop`
* **Variant:** `ParsingError::ExpectedConditionInWhile`
* **Resolution:** Provide a boolean condition inside parentheses: 'while (condition)'.

---

### `E0191`: Empty Enum Declaration
* **Diagnostic Message:** `Enum declaration must contain at least one variant`
* **Variant:** `ParsingError::EmptyEnumDeclaration`
* **Resolution:** Add at least one variant inside the enum body.

---

### `E0192`: Unexpected Semicolon
* **Diagnostic Message:** `Unexpected standalone semicolon outside statement context`
* **Variant:** `ParsingError::UnexpectedSemicolon`
* **Resolution:** Remove extraneous semicolon ';'.

---

### `E0193`: Missing Operator In Expression
* **Diagnostic Message:** `Consecutive expressions without an intervening operator`
* **Variant:** `ParsingError::MissingOperatorInExpression`
* **Resolution:** Add an operator between adjacent expressions or sub-expressions.

---

### `E0194`: Malformed Constructor Chaining
* **Diagnostic Message:** `Malformed constructor chaining invocation: expected 'super(...)' or 'this(...)'`
* **Variant:** `ParsingError::MalformedConstructorChaining`
* **Resolution:** Use ': super(...)' or ': this(...)' immediately following the constructor parameter list.

---

### `E0195`: Invalid Top Level Token
* **Diagnostic Message:** `Unexpected token at top level: '<token>'`
* **Variant:** `ParsingError::InvalidTopLevelToken`
* **Resolution:** Expected package, import, class, interface, enum, or function declaration at top level.

---

### `E0196`: Invalid Class Modifier
* **Diagnostic Message:** `Modifier '<modifier>' is not valid on class declarations`
* **Variant:** `ParsingError::InvalidClassModifier`
* **Resolution:** Valid class modifiers are: 'public', 'private', 'protected', 'internal', 'abstract', 'final'.

---

### `E0197`: Invalid Interface Modifier
* **Diagnostic Message:** `Modifier '<modifier>' is not valid on interface declarations`
* **Variant:** `ParsingError::InvalidInterfaceModifier`
* **Resolution:** Valid interface modifiers are: 'public', 'private', 'protected', 'internal'.

---

### `E0198`: Invalid Enum Modifier
* **Diagnostic Message:** `Modifier '<modifier>' is not valid on enum declarations`
* **Variant:** `ParsingError::InvalidEnumModifier`
* **Resolution:** Valid enum modifiers are: 'public', 'private', 'protected', 'internal'.

---

### `E0199`: Invalid Free Function Modifier
* **Diagnostic Message:** `Modifier '<modifier>' is not valid on package-level functions`
* **Variant:** `ParsingError::InvalidFreeFunctionModifier`
* **Resolution:** Valid free function modifiers are: 'public', 'private', 'internal', 'native', 'inline', 'extern'.

---

### `E0200`: Invalid Operator Modifier
* **Diagnostic Message:** `Modifier '<modifier>' is not valid on operator overload declarations`
* **Variant:** `ParsingError::InvalidOperatorModifier`
* **Resolution:** Valid operator modifiers are: 'public', 'inline'.
