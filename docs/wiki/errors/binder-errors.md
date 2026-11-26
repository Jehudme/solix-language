# Binder & Semantic Error Codes (`E0201` – `E0299`, `E0315` – `E0323`)

This section documents all semantic binding and type-checking diagnostics emitted by [`crates/solix-compilation/src/phases/binder/`](../../../crates/solix-compilation/src/phases/binder) and defined in [`BindingError`](../../../crates/solix-compilation/src/diagnostics/binding.rs).

---

### `E0201`: Undefined Symbol
* **Cause:** A variable, function, class, or method name is referenced without prior declaration or import.
* **Help:** Ensure the symbol is declared or imported into the current scope.
* **Bad Code:**
  ```solix
  int32 total = undefined_counter + 5;
  ```
* **Fixed Code:**
  ```solix
  int32 undefined_counter = 10;
  int32 total = undefined_counter + 5;
  ```

---

### `E0202`: Duplicate Symbol
* **Diagnostic Message:** `Duplicate symbol declaration '<symbol_name>'`
* **Variant:** `BindingError::DuplicateSymbol`
* **Resolution:** Choose a unique identifier or remove the redundant declaration.

---

### `E0203`: Type Mismatch
* **Cause:** Attempted to assign or supply a value whose type is incompatible with the expected target type.
* **Help:** Provide an expression compatible with the expected type or use an explicit cast.
* **Bad Code:**
  ```solix
  int32 number = "hello";
  ```
* **Fixed Code:**
  ```solix
  string message = "hello";
  ```

---

### `E0204`: Cyclic Class Inheritance
* **Cause:** A circular `extends` relationship was detected among class declarations.
* **Help:** Break the cyclic dependency in class inheritance.
* **Bad Code:**
  ```solix
  class NodeFirst extends NodeSecond {}
  class NodeSecond extends NodeFirst {}
  ```
* **Fixed Code:**
  ```solix
  class NodeBase {}
  class NodeDerived extends NodeBase {}
  ```

---

### `E0205`: Invalid Base Class
* **Diagnostic Message:** `Invalid superclass '<base_name>': <reason>`
* **Variant:** `BindingError::InvalidBaseClass`
* **Resolution:** Classes may only inherit from concrete or abstract classes, not interfaces or enums.

---

### `E0206`: Unimplemented Interface Method
* **Diagnostic Message:** `Class '<class_name>' does not implement interface method '<method_signature>' from '<interface_name>'`
* **Variant:** `BindingError::UnimplementedInterfaceMethod`
* **Resolution:** Implement all required interface methods in the concrete class.

---

### `E0207`: Method Override Mismatch
* **Diagnostic Message:** `Method override mismatch for '<method_name>' in superclass '<super_class>': <reason>`
* **Variant:** `BindingError::MethodOverrideMismatch`
* **Resolution:** Ensure the overriding method's parameter and return types match the superclass definition.

---

### `E0208`: Private Access Violation
* **Diagnostic Message:** `Private member '<member_name>' of class '<owner_class>' cannot be accessed from this scope`
* **Variant:** `BindingError::PrivateAccessViolation`
* **Resolution:** Private members can only be accessed within their declaring class.

---

### `E0209`: Protected Access Violation
* **Diagnostic Message:** `Protected member '<member_name>' of class '<owner_class>' cannot be accessed from this scope`
* **Variant:** `BindingError::ProtectedAccessViolation`
* **Resolution:** Protected members can only be accessed within the declaring class or its subclasses.

---

### `E0210`: Cannot Assign To Const
* **Diagnostic Message:** `Cannot assign to immutable or const variable '<variable_name>'`
* **Variant:** `BindingError::CannotAssignToConst`
* **Resolution:** Remove the 'const' modifier if this variable must be mutable.

---

### `E0211`: Return Outside Function
* **Diagnostic Message:** `Return statement outside function or method`
* **Variant:** `BindingError::ReturnOutsideFunction`
* **Resolution:** Move this return statement inside a function or method body.

---

### `E0212`: Non-Void Function Must Return A Value
* **Cause:** A non-void function or method execution path does not terminate with an explicit `return` statement or `throw` expression.
* **Help:** Ensure all code paths return an expression matching the declared return type.
* **Bad Code:**
  ```solix
  class Calculator {
      public static int32 compute(int32 operand) {
          if (operand > 0) {
              return operand * 2;
          }
      }
  }
  ```
* **Fixed Code:**
  ```solix
  class Calculator {
      public static int32 compute(int32 operand) {
          if (operand > 0) {
              return operand * 2;
          }
          return 0;
      }
  }
  ```

---

### `E0213`: Return Type Mismatch
* **Diagnostic Message:** `Return type mismatch: expected '<expected>', found '<found>'`
* **Variant:** `BindingError::ReturnTypeMismatch`
* **Resolution:** Change the returned expression to match the declared return type.

---

### `E0214`: Overload Ambiguity
* **Diagnostic Message:** `Ambiguous method call '<method_name>' for arguments '(<argument_types>)'`
* **Variant:** `BindingError::OverloadAmbiguity`
* **Resolution:** Disambiguate the call with explicit type casts.

---

### `E0215`: No Matching Overload
* **Diagnostic Message:** `No matching overload for '<method_name>' with arguments '(<argument_types>)'`
* **Variant:** `BindingError::NoMatchingOverload`
* **Resolution:** Verify method argument types or provide a suitable overload.

---

### `E0216`: Generic Arity Mismatch
* **Diagnostic Message:** `Generic arity mismatch for '<template_name>': expected <expected> type arguments, got <found>`
* **Variant:** `BindingError::GenericArityMismatch`
* **Resolution:** Provide the exact number of type arguments expected by the generic definition.

---

### `E0217`: Cannot Instantiate Abstract Type
* **Diagnostic Message:** `Cannot instantiate abstract class or interface '<type_name>'`
* **Variant:** `BindingError::CannotInstantiateAbstractType`
* **Resolution:** Instantiate a concrete subclass that implements all required methods.

---

### `E0218`: Super Constructor Must Be First
* **Diagnostic Message:** `Super constructor call ': super(...)' must be the first statement in constructor body`
* **Variant:** `BindingError::SuperConstructorMustBeFirst`
* **Resolution:** Invoke superclass constructors before executing any subclass logic.

---

### `E0219`: Break Or Continue Outside Loop
* **Diagnostic Message:** `Break or continue statement outside of loop`
* **Variant:** `BindingError::BreakOrContinueOutsideLoop`
* **Resolution:** Break and continue statements can only appear within loop bodies.

---

### `E0220`: Invalid Operator Overload
* **Diagnostic Message:** `Invalid operator overload '<operator_name>': <reason>`
* **Variant:** `BindingError::InvalidOperatorOverload`
* **Resolution:** Binary operators require 1 parameter; unary operators require 0 parameters.

---

### `E0221`: Non Indexable Type
* **Diagnostic Message:** `Type '<found>' cannot be indexed with array brackets`
* **Variant:** `BindingError::NonIndexableType`
* **Resolution:** Array subscript indexing is only valid on arrays and tuples.

---

### `E0222`: Non Callable Type
* **Diagnostic Message:** `Type '<found>' cannot be invoked as a function`
* **Variant:** `BindingError::NonCallableType`
* **Resolution:** Only function pointers, methods, and closures can be invoked.

---

### `E0223`: Cannot Infer Type
* **Diagnostic Message:** `Cannot infer type for expression`
* **Variant:** `BindingError::CannotInferType`
* **Resolution:** Provide an explicit type annotation.

---

### `E0224`: Ambiguous Import
* **Diagnostic Message:** `Ambiguous imported symbol '<symbol_name>'; matches candidates: <value>`
* **Variant:** `BindingError::AmbiguousImport`
* **Resolution:** Disambiguate by importing or referencing the symbol with its fully-qualified package prefix.

---

### `E0225`: Non Constant Case Value
* **Diagnostic Message:** `Case switch label must be a compile-time constant expression`
* **Variant:** `BindingError::NonConstantCaseValue`
* **Resolution:** Use literal constants or enum variants as case values.

---

### `E0226`: Static Access Via Dot Operator
* **Cause:** A static method or field was accessed using the instance member access operator (`.`) instead of scope resolution (`::`).
* **Help:** Replace '.' with '::' to access static members or enum variants.
* **Bad Code:**
  ```solix
  import solix.math.Math;
  float64 root = Math.sqrt(16.0);
  ```
* **Fixed Code:**
  ```solix
  import solix.math.Math;
  float64 root = Math::sqrt(16.0);
  ```

---

### `E0227`: Instance Access Via Scope Resolution
* **Diagnostic Message:** `Instance member '<member_name>' of '<owner>' must be accessed using '.' instead of '::'`
* **Variant:** `BindingError::InstanceAccessViaScopeResolution`
* **Resolution:** Replace '::' with '.' to access instance fields or methods.

---

### `E0228`: Invalid Super Call
* **Diagnostic Message:** `Keyword 'super' cannot be used in a class with no superclass`
* **Variant:** `BindingError::InvalidSuperCall`
* **Resolution:** Remove the ': super(...)' invocation or specify a base class with 'extends'.

---

### `E0229`: Invalid Weak Modifier
* **Diagnostic Message:** `Modifier 'weak' can only be applied to reference types, found '<found>'`
* **Variant:** `BindingError::InvalidWeakModifier`
* **Resolution:** Remove 'weak' or change the field to a class or interface reference type.

---

### `E0230`: Circular Alias
* **Diagnostic Message:** `Circular alias detected in '<alias_name>'`
* **Variant:** `BindingError::CircularAlias`
* **Resolution:** Break the cyclic alias chain.

---

### `E0231`: Unresolved Alias Target
* **Diagnostic Message:** `Cannot resolve alias target '<target_name>'`
* **Variant:** `BindingError::UnresolvedAliasTarget`
* **Resolution:** Ensure the aliased type exists and is in scope.

---

### `E0232`: Cannot Resolve Import
* **Diagnostic Message:** `Cannot resolve import '<import_path>': package or symbol not found`
* **Variant:** `BindingError::CannotResolveImport`
* **Resolution:** Ensure the imported package and symbol exist.

---

### `E0233`: Duplicate Local Variable
* **Cause:** A local variable was declared with a name already in use within the current block scope.
* **Help:** Rename the variable to avoid shadowing or duplicate declarations in the same scope.
* **Bad Code:**
  ```solix
  int32 index = 0;
  int32 index = 1;
  ```
* **Fixed Code:**
  ```solix
  int32 first_index = 0;
  int32 second_index = 1;
  ```

---

### `E0234`: Duplicate Parameter Name
* **Diagnostic Message:** `Duplicate parameter name '<name>' in parameter list`
* **Variant:** `BindingError::DuplicateParameterName`
* **Resolution:** Parameter names within a declaration must be unique.

---

### `E0235`: Cannot Inherit From Final Class
* **Cause:** Attempted to extend a class that is declared with the `final` modifier.
* **Help:** Final classes cannot be extended.
* **Bad Code:**
  ```solix
  final class ImmutableBase {}
  class Derived extends ImmutableBase {}
  ```
* **Fixed Code:**
  ```solix
  class OpenBase {}
  class Derived extends OpenBase {}
  ```

---

### `E0236`: Superclass Must Be Class
* **Diagnostic Message:** `Type '<found>' cannot be used as a superclass; expected a class`
* **Variant:** `BindingError::SuperclassMustBeClass`
* **Resolution:** Only classes can be extended via 'extends'.

---

### `E0237`: Implemented Type Must Be Interface
* **Diagnostic Message:** `Type '<found>' cannot be used in 'implements' clause; expected an interface`
* **Variant:** `BindingError::ImplementedTypeMustBeInterface`
* **Resolution:** Only interfaces can be implemented via 'implements'.

---

### `E0238`: Argument Count Mismatch
* **Cause:** A function or constructor was invoked with a different number of arguments than its declared parameter list.
* **Help:** Provide the exact number of required arguments.
* **Bad Code:**
  ```solix
  int32 add(int32 first, int32 second) { return first + second; }
  add(1);
  ```
* **Fixed Code:**
  ```solix
  add(1, 2);
  ```

---

### `E0239`: Argument Type Mismatch
* **Diagnostic Message:** `Argument <parameter_index> type mismatch: expected '<expected>', found '<found>'`
* **Variant:** `BindingError::ArgumentTypeMismatch`
* **Resolution:** Pass an argument matching the expected parameter type or use an explicit cast.

---

### `E0240`: Field Initializer Type Mismatch
* **Diagnostic Message:** `Field '<field_name>' initializer type mismatch: expected '<expected>', found '<found>'`
* **Variant:** `BindingError::FieldInitializerTypeMismatch`
* **Resolution:** Provide an initializer expression matching the declared field type.

---

### `E0241`: Member Access On Non Composite Type
* **Diagnostic Message:** `Cannot access member '<member_name>' on non-composite type '<found>'`
* **Variant:** `BindingError::MemberAccessOnNonCompositeType`
* **Resolution:** Member access '.' is only valid on classes, interfaces, or structs.

---

### `E0242`: Super Constructor Argument Mismatch
* **Diagnostic Message:** `Super constructor expects <expected> argument(s), but <found> were provided`
* **Variant:** `BindingError::SuperConstructorArgumentMismatch`
* **Resolution:** Provide arguments matching the base class constructor definition.

---

### `E0243`: Main Method Signature Invalid
* **Diagnostic Message:** `Invalid entry point signature: <reason>`
* **Variant:** `BindingError::MainMethodSignatureInvalid`
* **Resolution:** Entry point functions (default 'main', or configured via 'profiles.<profile>.entrypoint' or '--entry') must have return type 'int32' or 'void' and accept () or (string[] / char[][]).

---

### `E0244`: Multiple Main Methods Found
* **Diagnostic Message:** `Multiple 'main' entry point functions discovered (<count> found across package)`
* **Variant:** `BindingError::MultipleMainMethodsFound`
* **Resolution:** Define a single unique 'main' entry point function, or select an explicit entry point via 'profiles.<profile>.entrypoint' or '--entry'.

---

### `E0245`: Duplicate Enum Variant
* **Diagnostic Message:** `Duplicate variant '<variant_name>' in enum '<enum_name>'`
* **Variant:** `BindingError::DuplicateEnumVariant`
* **Resolution:** Each variant identifier in an enum must be unique.

---

### `E0246`: Enum Explicit Value Not Integer
* **Diagnostic Message:** `Explicit value for enum variant '<variant_name>' must be an integer, found '<found>'`
* **Variant:** `BindingError::EnumExplicitValueNotInteger`
* **Resolution:** Enum variant values must evaluate to integer constants.

---

### `E0247`: Unary Operator Type Mismatch
* **Diagnostic Message:** `Unary operator '<operator>' cannot be applied to operand of type '<operand_type>'`
* **Variant:** `BindingError::UnaryOperatorTypeMismatch`
* **Resolution:** Apply the unary operator to a compatible operand type.

---

### `E0248`: Binary Operator Type Mismatch
* **Cause:** A binary operator is applied to incompatible operand types that lack an overloaded operator.
* **Help:** Apply the binary operator to compatible operand types.
* **Bad Code:**
  ```solix
  int32 numeric_value = 10;
  string text_value = "hello";
  bool comparison_result = numeric_value < text_value;
  ```
* **Fixed Code:**
  ```solix
  int32 numeric_value = 10;
  int32 boundary_limit = 20;
  bool comparison_result = numeric_value < boundary_limit;
  ```

---

### `E0249`: Logical Condition Not Boolean
* **Cause:** An `if`, `while`, or `do`/`while` condition expression evaluated to a non-boolean type.
* **Help:** Ensure loop or if conditions produce boolean values.
* **Bad Code:**
  ```solix
  int32 count = 5;
  if (count) { }
  ```
* **Fixed Code:**
  ```solix
  int32 count = 5;
  if (count > 0) { }
  ```

---

### `E0250`: Array Length Not Integer
* **Diagnostic Message:** `Array size expression must evaluate to an integer type, found '<found>'`
* **Variant:** `BindingError::ArrayLengthNotInteger`
* **Resolution:** Specify array sizes using integer expressions.

---

### `E0251`: Index Expression Not Integer
* **Diagnostic Message:** `Array index expression must evaluate to an integer type, found '<found>'`
* **Variant:** `BindingError::IndexExpressionNotInteger`
* **Resolution:** Subscript index expressions must evaluate to integers.

---

### `E0252`: Tuple Index Out Of Range
* **Diagnostic Message:** `Tuple index <index> out of range for tuple of size <size>`
* **Variant:** `BindingError::TupleIndexOutOfRange`
* **Resolution:** Tuple indices are 0-based and must be less than the tuple element count.

---

### `E0253`: Cannot Assign To Read-Only Target
* **Cause:** Attempted to reassign an intrinsic read-only reference (`this`, `super`), or assign to a non-assignable symbol.
* **Help:** Target is read-only and cannot be modified.
* **Bad Code:**
  ```solix
  class State {
      public void execute() {}
      public void replace() {
          this = new State();
      }
  }
  ```
* **Fixed Code:**
  ```solix
  class State {
      public int32 counter = 0;
      public void update(int32 next_value) {
          this.counter = next_value;
      }
  }
  ```

---

### `E0254`: Void Type Not Allowed Here
* **Diagnostic Message:** `Type 'void' is only valid as a return type and cannot be used here`
* **Variant:** `BindingError::VoidTypeNotAllowedHere`
* **Resolution:** Variables, parameters, and fields cannot be of type 'void'.

---

### `E0255`: Unknown Type In Declaration
* **Diagnostic Message:** `Unknown type '<type_name>' in declaration`
* **Variant:** `BindingError::UnknownTypeInDeclaration`
* **Resolution:** Ensure the declared type name exists and is imported.

---

### `E0256`: Duplicate Field Declaration
* **Diagnostic Message:** `Duplicate field '<field_name>' in class '<class_name>'`
* **Variant:** `BindingError::DuplicateFieldDeclaration`
* **Resolution:** Field names within a class must be unique.

---

### `E0257`: Duplicate Method Declaration
* **Diagnostic Message:** `Duplicate method '<method_signature>' in class '<class_name>'`
* **Variant:** `BindingError::DuplicateMethodDeclaration`
* **Resolution:** Method signatures within a class must be unique.

---

### `E0258`: Duplicate Constructor Declaration
* **Diagnostic Message:** `Duplicate constructor signature '<constructor_signature>'`
* **Variant:** `BindingError::DuplicateConstructorDeclaration`
* **Resolution:** Constructor overloads must have distinct parameter lists.

---

### `E0259`: Constructor Chaining Cycle
* **Diagnostic Message:** `Constructor chaining cycle detected in constructor '<constructor_name>'`
* **Variant:** `BindingError::ConstructorChainingCycle`
* **Resolution:** Avoid recursive ': this(...)' constructor delegations.

---

### `E0260`: This Used Outside Instance Method
* **Diagnostic Message:** `Keyword 'this' can only be used inside instance methods, constructors, or destructors`
* **Variant:** `BindingError::ThisUsedOutsideInstanceMethod`
* **Resolution:** Remove 'this' from static or package-level contexts.

---

### `E0261`: Super Used Outside Subclass
* **Diagnostic Message:** `Keyword 'super' can only be used inside subclasses with a declared superclass`
* **Variant:** `BindingError::SuperUsedOutsideSubclass`
* **Resolution:** Ensure the enclosing class extends a base class before using 'super'.

---

### `E0262`: Static Method Access Via Instance
* **Diagnostic Message:** `Static method '<method_name>' of class '<class_name>' should be invoked via '::' rather than an instance`
* **Variant:** `BindingError::StaticMethodAccessViaInstance`
* **Resolution:** Invoke static methods using the class name: 'ClassName::method()'.

---

### `E0263`: Instance Method Access Via Static
* **Diagnostic Message:** `Instance method '<method_name>' of class '<class_name>' cannot be invoked statically without an instance`
* **Variant:** `BindingError::InstanceMethodAccessViaStatic`
* **Resolution:** Instance methods require an object instance to be invoked.

---

### `E0264`: Abstract Method in Non-Abstract Class
* **Cause:** A method was declared `abstract` inside a concrete class declaration. Only `abstract` classes or interfaces may declare abstract methods.
* **Help:** Declare the enclosing class 'abstract' or provide a method body.
* **Bad Code:**
  ```solix
  class Worker {
      public abstract void execute();
  }
  ```
* **Fixed Code:**
  ```solix
  abstract class Worker {
      public abstract void execute();
  }
  ```

---

### `E0265`: Non Abstract Class Missing Abstract Impl
* **Diagnostic Message:** `Class '<class_name>' must either be declared abstract or implement abstract method '<missing_method>'`
* **Variant:** `BindingError::NonAbstractClassMissingAbstractImpl`
* **Resolution:** Implement all inherited abstract methods or mark the class 'abstract'.

---

### `E0266`: Interface Field Not Constant
* **Diagnostic Message:** `Fields in interfaces must be compile-time constants`
* **Variant:** `BindingError::InterfaceFieldNotConstant`
* **Resolution:** Declare interface fields 'const' with compile-time literal initializers.

---

### `E0267`: Final Method Overridden
* **Cause:** Attempted to override a method that is declared with the `final` modifier in a base class.
* **Help:** Final methods cannot be overridden in derived classes.
* **Bad Code:**
  ```solix
  class Base {
      public final int32 compute() { return 1; }
  }
  class Derived extends Base {
      public override int32 compute() { return 2; }
  }
  ```
* **Fixed Code:**
  ```solix
  class Base {
      public virtual int32 compute() { return 1; }
  }
  class Derived extends Base {
      public override int32 compute() { return 2; }
  }
  ```

---

### `E0268`: Narrowing Visibility In Override
* **Diagnostic Message:** `Cannot reduce visibility of overridden method '<method_name>' from '<original_visibility>' to '<new_visibility>'`
* **Variant:** `BindingError::NarrowingVisibilityInOverride`
* **Resolution:** Overriding methods cannot declare more restrictive visibility than the parent method.

---

### `E0269`: Invalid Cast Type
* **Diagnostic Message:** `Cannot cast expression of type '<source_type>' to '<target_type>' (unrelated types)`
* **Variant:** `BindingError::InvalidCastType`
* **Resolution:** Ensure types are related by inheritance or numeric conversion paths.

---

### `E0270`: Invalid Type For Instance Of
* **Diagnostic Message:** `Right-hand operand of 'instanceof' must be a class or interface type, found '<found>'`
* **Variant:** `BindingError::InvalidTypeForInstanceOf`
* **Resolution:** Use 'instanceof' only with nominal class or interface types.

---

### `E0271`: Dead Code After Return Or Throw
* **Diagnostic Message:** `Unreachable statement detected after return or throw`
* **Variant:** `BindingError::DeadCodeAfterReturnOrThrow`
* **Resolution:** Remove unreachable statements after unconditional control flow divergence.

---

### `E0272`: Uninitialized Variable Used
* **Diagnostic Message:** `Local variable '<variable_name>' used before being initialized`
* **Variant:** `BindingError::UninitializedVariableUsed`
* **Resolution:** Initialize local variables before reading their value.

---

### `E0273`: Duplicate Access Modifier
* **Diagnostic Message:** `Duplicate access modifier '<modifier>'`
* **Variant:** `BindingError::DuplicateAccessModifier`
* **Resolution:** Remove duplicate access modifiers.

---

### `E0274`: Conflicting Modifier Combination
* **Diagnostic Message:** `Cannot combine '<modifier1>' and '<modifier2>' modifiers`
* **Variant:** `BindingError::ConflictingModifierCombination`
* **Resolution:** Remove one of the conflicting modifiers.

---

### `E0275`: Modifier Not Applicable To Declaration
* **Diagnostic Message:** `Modifier '<modifier>' is not applicable to <declaration_kind> declarations`
* **Variant:** `BindingError::ModifierNotApplicableToDeclaration`
* **Resolution:** Remove modifier invalid for this declaration kind.

---

### `E0276`: Virtual On Non Method
* **Diagnostic Message:** `Modifier 'virtual' can only be applied to methods inside a class`
* **Variant:** `BindingError::VirtualOnNonMethod`
* **Resolution:** Remove 'virtual'; only class methods can be virtual.

---

### `E0277`: Override On Non Method
* **Diagnostic Message:** `Modifier 'override' can only be applied to methods inside a class`
* **Variant:** `BindingError::OverrideOnNonMethod`
* **Resolution:** Remove 'override'; only class methods can be overrides.

---

### `E0278`: Native On Non Method Or Function
* **Diagnostic Message:** `Modifier 'native' can only be applied to functions or methods`
* **Variant:** `BindingError::NativeOnNonMethodOrFunction`
* **Resolution:** Remove 'native'; only callable declarations can be native.

---

### `E0279`: Inline On Non Method Or Function
* **Diagnostic Message:** `Modifier 'inline' can only be applied to functions or methods`
* **Variant:** `BindingError::InlineOnNonMethodOrFunction`
* **Resolution:** Remove 'inline'; only callable declarations can be inline.

---

### `E0280`: Static On Constructor Or Destructor
* **Diagnostic Message:** `Modifier 'static' cannot be applied to constructors or destructors`
* **Variant:** `BindingError::StaticOnConstructorOrDestructor`
* **Resolution:** Constructors and destructors cannot be declared static.

---

### `E0281`: Visibility On Destructor
* **Diagnostic Message:** `Access modifiers cannot be applied to destructors`
* **Variant:** `BindingError::VisibilityOnDestructor`
* **Resolution:** Destructors have no visibility modifier; remove 'public', 'private', etc.

---

### `E0282`: Override Without Base Method
* **Diagnostic Message:** `Method '<method_name>' marked 'override' does not override any base class method`
* **Variant:** `BindingError::OverrideWithoutBaseMethod`
* **Resolution:** Ensure the method signature matches a virtual method in a superclass.

---

### `E0283`: Override Non Virtual Method
* **Diagnostic Message:** `Cannot override method '<method_name>' of class '<base_class>' because it is not declared 'virtual'`
* **Variant:** `BindingError::OverrideNonVirtualMethod`
* **Resolution:** Declare the base class method 'virtual' or remove 'override'.

---

### `E0284`: Direct Destructor Invocation
* **Cause:** Destructors are managed automatically by ARC and cannot be invoked explicitly using member access syntax.
* **Help:** Remove explicit destructor call; ARC cleans up objects automatically.
* **Bad Code:**
  ```solix
  Buffer buf = new Buffer();
  buf.~Buffer();
  ```
* **Fixed Code:**
  ```solix
  Buffer buf = new Buffer();
  buf.clear();
  ```

---

### `E0285`: Extern Symbol Missing Signature
* **Diagnostic Message:** `Extern symbol '<symbol_name>' is missing an explicit type signature`
* **Variant:** `BindingError::ExternSymbolMissingSignature`
* **Resolution:** Provide explicit types for extern functions and variables.

---

### `E0286`: Parameter Default Value Mismatch
* **Diagnostic Message:** `Default value for parameter '<parameter_name>' type mismatch: expected '<expected>', found '<found>'`
* **Variant:** `BindingError::ParameterDefaultValueMismatch`
* **Resolution:** Ensure default parameter expression matches declared parameter type.

---

### `E0287`: Invalid Expression in Constant Context
* **Cause:** A `const` declaration initializer contains a non-constant expression such as a dynamic call or runtime allocation.
* **Help:** Only literal constants, basic arithmetic, and const references are allowed in constant expressions.
* **Bad Code:**
  ```solix
  class Configuration {
      public const int32 DYNAMIC_PORT = NetworkService::resolve_port();
  }
  ```
* **Fixed Code:**
  ```solix
  class Configuration {
      public const int32 FIXED_PORT = 8080;
  }
  ```

---

### `E0288`: Bitwise Operator on Non-Integer
* **Cause:** A bitwise operator (`&`, `|`, `^`, `<<`, `>>`) was applied to a floating-point or reference type.
* **Help:** Bitwise operators ('&', '|', '^', '<<', '>>', '~') are only valid on integer types.
* **Bad Code:**
  ```solix
  float64 mask = 3.14 & 1.0;
  ```
* **Fixed Code:**
  ```solix
  int32 mask = 14 & 1;
  ```

---

### `E0289`: Incompatible Dependency Version
* **Diagnostic Message:** `Incompatible dependency version for package '<name>': <reason>`
* **Variant:** `BindingError::IncompatibleDependencyVersion`
* **Resolution:** Ensure that all dependencies in the tree require compatible versions (same major, >= medium).

---

### `E0290`: External Package Not Found
* **Diagnostic Message:** `External package '<name>' not found: <reason>`
* **Variant:** `BindingError::ExternalPackageNotFound`
* **Resolution:** Install the package using 'solix install' or verify the package name.

---

### `E0291`: Dependency Checksum Mismatch
* **Diagnostic Message:** `Dependency checksum verification failed for '<name>@<version>': expected <expected_sha256>, found <actual_sha256>`
* **Variant:** `BindingError::DependencyChecksumMismatch`
* **Resolution:** The resolved package artifact does not match the pinned compile-time SHA-256 digest.\n\ This indicates the package has been mutated, corrupted, or tampered with.\n\ To bypass checksum enforcement when testing modified dependencies, pass '--no-verify'.

---

### `E0292`: Switch Class Missing Equality Operator
* **Diagnostic Message:** `Class '<class_name>' does not implement 'operator ==' or 'operator !=' and cannot be used in equality switch cases.`
* **Variant:** `BindingError::SwitchClassMissingEqualityOperator`
* **Resolution:** Implement 'operator ==' or 'operator !=' in class '{class_name}' to support switch equality dispatch.

---

### `E0293`: Incompatible Switch Case Type
* **Diagnostic Message:** `Case expression of type '<case_type>' is incompatible with switch discriminant of type '<discriminant_type>'.`
* **Variant:** `BindingError::IncompatibleSwitchCaseType`
* **Resolution:** Ensure the case expression type matches or is assignable to the switch discriminant type.

---

### `E0294`: Duplicate Switch Case Constant
* **Diagnostic Message:** `Duplicate case value '<case_value>' detected in switch statement.`
* **Variant:** `BindingError::DuplicateSwitchCaseConstant`
* **Resolution:** Remove or combine duplicate case branches.

---

### `E0295`: Invalid Switch Pattern Target
* **Diagnostic Message:** `Right-hand target of 'instanceof' in switch case must be a class, interface, or tuple type, found '<found>'.`
* **Variant:** `BindingError::InvalidSwitchPatternTarget`
* **Resolution:** Use nominal class, interface, or tuple type in 'case instanceof <Type>'.

---

### `E0296`: Tuple Arity Mismatch In Case
* **Diagnostic Message:** `Tuple case pattern arity <case_arity> does not match switch discriminant tuple arity <disc_arity>.`
* **Variant:** `BindingError::TupleArityMismatchInCase`
* **Resolution:** Ensure the tuple pattern has the same number of elements as the discriminant tuple.

---

### `E0297`: Generic Type Constraint Violation
* **Cause:** A type argument supplied for a generic parameter does not satisfy the declared constraint bound.
* **Help:** Ensure the provided type argument implements or inherits the required bound.
* **Bad Code:**
  ```solix
  class Container<ItemType extends Component> {}
  Container<int32> container;
  ```
* **Fixed Code:**
  ```solix
  class Container<ItemType extends Component> {}
  Container<Widget> container;
  ```

---

### `E0298`: Cannot Throw Null Literal
* **Cause:** A `throw` statement directly specifies `null` or an expression statically typed as `null`.
* **Help:** Throw expressions must evaluate to a non-null instance of an Exception class.
* **Bad Code:**
  ```solix
  void faulty() { throw null; }
  ```
* **Fixed Code:**
  ```solix
  class CustomException extends Exception {}
  void faulty() { throw new CustomException(); }
  ```

---

### `E0299`: Negative Array Length
* **Cause:** An array creation expression specifies a compile-time constant negative dimension.
* **Help:** Array length expressions must evaluate to a non-negative integer.
* **Bad Code:**
  ```solix
  int32[] buffer = new int32[-10];
  ```
* **Fixed Code:**
  ```solix
  int32[] buffer = new int32[10];
  ```

---

### `E0315`: Catch Clause Type Not Exception Subclass
* **Cause:** A `catch` clause specifies a class that does not inherit from `solix.core.Exception` (or `Exception`).
* **Help:** Catch clauses can only target classes that extend 'solix.core.Exception'.
* **Bad Code:**
  ```solix
  try { } catch (PlainRecord error_record) { }
  ```
* **Fixed Code:**
  ```solix
  try { } catch (CustomException custom_error) { }
  ```

---

### `E0316`: Throw Statement Type Not Exception Subclass
* **Cause:** A `throw` statement specifies an instance that does not inherit from `solix.core.Exception` (or `Exception`).
* **Help:** Throw statements can only throw classes that extend 'solix.core.Exception'.
* **Bad Code:**
  ```solix
  throw plain_record;
  ```
* **Fixed Code:**
  ```solix
  throw new CustomException("invalid state");
  ```

---

### `E0317`: Assertion Message Expression Not String
* **Cause:** The custom failure message in an `assert(condition, message)` statement does not evaluate to type `string`.
* **Help:** Provide an expression that evaluates to a string for the assertion failure message.
* **Bad Code:**
  ```solix
  assert(value > 0, 404);
  ```
* **Fixed Code:**
  ```solix
  assert(value > 0, "Value must be positive");
  ```

---

### `E0318`: Generic Instantiation Depth Exceeded
* **Cause:** Monomorphization depth of a generic class or self-referential generic expansion exceeded the recursion limit.
* **Help:** Avoid self-referential or infinitely expanding recursive generic definitions.
* **Bad Code:**
  ```solix
  class RecursiveNode<ItemType> {
      public RecursiveNode<RecursiveNode<ItemType>> next_child;
  }
  ```
* **Fixed Code:**
  ```solix
  class RecursiveNode<ItemType> {
      public RecursiveNode<ItemType> next_child;
  }
  ```

---

### `E0319`: Cyclic Static Field Dependency
* **Cause:** Static field initializers form a cyclic initialization dependency between classes or fields.
* **Help:** Break the circular dependency between static field initializers.
* **Bad Code:**
  ```solix
  class FirstClass { public static int32 first_val = SecondClass::second_val + 1; }
  class SecondClass { public static int32 second_val = FirstClass::first_val + 1; }
  ```
* **Fixed Code:**
  ```solix
  class FirstClass { public static int32 first_val = 10; }
  class SecondClass { public static int32 second_val = FirstClass::first_val + 1; }
  ```

---

### `E0320`: Virtual Call in Destructor Forbidden
* **Cause:** Calling a virtual or overridden instance method on `this` inside a destructor is forbidden to prevent torn dispatch.
* **Help:** Only non-virtual, private, or static methods can be safely called during object destruction.
* **Bad Code:**
  ```solix
  class Resource {
      public virtual void teardown() {}
      ~Resource() { this.teardown(); }
  }
  ```
* **Fixed Code:**
  ```solix
  class Resource {
      public void teardown_non_virtual() {}
      ~Resource() { this.teardown_non_virtual(); }
  }
  ```

---

### `E0321`: Symbol Collides with Package Name
* **Cause:** A top-level class, interface, or enum declaration identifier collides with a namespace segment of a declared package.
* **Help:** Rename the top-level declaration to avoid collisions with package namespace segments.
* **Bad Code:**
  ```solix
  package network.http;
  class Client {}
  // Elsewhere: class network {}
  ```
* **Fixed Code:**
  ```solix
  class NetworkManager {}
  ```

---

### `E0322`: Inexhaustive Enum Switch
* **Cause:** A switch statement on an enum type does not cover all declared variants and lacks a `default` branch.
* **Help:** Provide an explicit 'default' case to safely handle arbitrary or cast enum values.
* **Bad Code:**
  ```solix
  enum State { Pending, Active, Completed }
  switch (current_state) {
      case State::Pending: return 1;
  }
  ```
* **Fixed Code:**
  ```solix
  enum State { Pending, Active, Completed }
  switch (current_state) {
      case State::Pending: return 1;
      case State::Active: return 2;
      case State::Completed: return 3;
  }
  ```

---

### `E0323`: Cyclic Interface Inheritance
* **Cause:** An inheritance cycle was detected in interface `extends` declarations.
* **Help:** Break the cyclic dependency in interface inheritance.
* **Bad Code:**
  ```solix
  interface AlphaProtocol extends BetaProtocol {}
  interface BetaProtocol extends AlphaProtocol {}
  ```
* **Fixed Code:**
  ```solix
  interface AlphaProtocol {}
  interface BetaProtocol extends AlphaProtocol {}
  ```
