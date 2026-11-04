//! Granular Semantic Negative Test Suite targeting all semantic binder diagnostic codes (E0201..E0288).
//!
//! Asserts exact error codes with `assert_compile_error(source, "E02XX")` and verifies
//! that no internal compiler representations leak to diagnostics.

#![forbid(unsafe_code)]

use solix_tests::common::assert_compile_error;

// =============================================================================
// E0201: Undefined Symbol
// =============================================================================

#[test]
fn test_negative_undefined_symbol_variable() {
    let source_code = r#"
        void main() {
            int32 current_value = missing_variable;
        }
    "#;
    assert_compile_error(source_code, "E0201");
}

#[test]
fn test_negative_undefined_symbol_function_call() {
    let source_code = r#"
        void main() {
            unknown_subroutine();
        }
    "#;
    assert_compile_error(source_code, "E0201");
}

#[test]
fn test_negative_undefined_symbol_member() {
    let source_code = r#"
        class Account {
            public int32 balance = 100;
        }

        void main() {
            Account account = new Account();
            int32 pending_balance = account.non_existent_member;
        }
    "#;
    assert_compile_error(source_code, "E0201");
}

// =============================================================================
// E0202: Duplicate Symbol
// =============================================================================

#[test]
fn test_negative_duplicate_symbol_top_level_class() {
    let source_code = r#"
        class DuplicateClass {}
        class DuplicateClass {}
    "#;
    assert_compile_error(source_code, "E0202");
}

#[test]
fn test_negative_duplicate_symbol_class_field() {
    let source_code = r#"
        class Container {
            public int32 field_name = 1;
            public int32 field_name = 2;
        }
    "#;
    assert_compile_error(source_code, "E0202");
}

#[test]
fn test_negative_duplicate_symbol_local_variable() {
    let source_code = r#"
        void main() {
            int32 duplicate_var = 10;
            int32 duplicate_var = 20;
        }
    "#;
    assert_compile_error(source_code, "E0202");
}

#[test]
fn test_negative_duplicate_symbol_function_parameter() {
    let source_code = r#"
        void compute(int32 duplicate_param, float64 duplicate_param) {}
    "#;
    assert_compile_error(source_code, "E0202");
}

// =============================================================================
// E0203: Type Mismatch
// =============================================================================

#[test]
fn test_negative_type_mismatch_variable_initialization() {
    let source_code = r#"
        void main() {
            int32 integer_target = true;
        }
    "#;
    assert_compile_error(source_code, "E0203");
}

#[test]
fn test_negative_type_mismatch_assignment() {
    let source_code = r#"
        void main() {
            int32 integer_target = 42;
            integer_target = "hello string";
        }
    "#;
    assert_compile_error(source_code, "E0203");
}

#[test]
fn test_negative_type_mismatch_if_condition_not_boolean() {
    let source_code = r#"
        void main() {
            if (12345) {
                int32 placeholder = 1;
            }
        }
    "#;
    assert_compile_error(source_code, "E0203");
}

#[test]
fn test_negative_type_mismatch_while_condition_not_boolean() {
    let source_code = r#"
        void main() {
            while ("not a boolean") {
                int32 placeholder = 1;
            }
        }
    "#;
    assert_compile_error(source_code, "E0203");
}

// =============================================================================
// E0204: Cyclic Inheritance
// =============================================================================

#[test]
fn test_negative_cyclic_inheritance_direct() {
    let source_code = r#"
        class SelfInheriting extends SelfInheriting {}
    "#;
    assert_compile_error(source_code, "E0204");
}

#[test]
fn test_negative_cyclic_inheritance_mutual() {
    let source_code = r#"
        class NodeA extends NodeB {}
        class NodeB extends NodeA {}
    "#;
    assert_compile_error(source_code, "E0204");
}

// =============================================================================
// E0205: Invalid Base Class
// =============================================================================

#[test]
fn test_negative_invalid_base_class_primitive() {
    let source_code = r#"
        class ChildFromPrimitive extends int32 {}
    "#;
    assert_compile_error(source_code, "E0205");
}

// =============================================================================
// E0206: Unimplemented Interface Method
// =============================================================================

#[test]
fn test_negative_unimplemented_interface_method() {
    let source_code = r#"
        interface IWorker {
            void execute_task();
        }

        class ConcreteWorker implements IWorker {}
    "#;
    assert_compile_error(source_code, "E0206");
}

// =============================================================================
// E0207: Method Override Mismatch
// =============================================================================

#[test]
fn test_negative_method_override_mismatch_signature() {
    let source_code = r#"
        class BaseHandler {
            public virtual void handle(int32 event_code) {}
        }

        class SpecializedHandler extends BaseHandler {
            public override void handle(float64 event_code) {}
        }
    "#;
    assert_compile_error(source_code, "E0207");
}

// =============================================================================
// E0208: Private Access Violation
// =============================================================================

#[test]
fn test_negative_private_access_violation_field() {
    let source_code = r#"
        class Vault {
            private int32 secret_combination = 1234;
        }

        void main() {
            Vault vault_instance = new Vault();
            int32 stolen_secret = vault_instance.secret_combination;
        }
    "#;
    assert_compile_error(source_code, "E0208");
}

#[test]
fn test_negative_private_access_violation_method() {
    let source_code = r#"
        class Vault {
            private void open_door() {}
        }

        void main() {
            Vault vault_instance = new Vault();
            vault_instance.open_door();
        }
    "#;
    assert_compile_error(source_code, "E0208");
}

// =============================================================================
// E0209: Protected Access Violation
// =============================================================================

#[test]
fn test_negative_protected_access_violation_from_unrelated_class() {
    let source_code = r#"
        class BaseEntity {
            protected int32 internal_id = 999;
        }

        class UnrelatedAuditor {
            void inspect() {
                BaseEntity entity = new BaseEntity();
                int32 extracted_id = entity.internal_id;
            }
        }
    "#;
    assert_compile_error(source_code, "E0209");
}

// =============================================================================
// E0210: Cannot Assign to Const
// =============================================================================

#[test]
fn test_negative_cannot_assign_to_const_local() {
    let source_code = r#"
        void main() {
            const int32 fixed_constant = 100;
            fixed_constant = 200;
        }
    "#;
    assert_compile_error(source_code, "E0210");
}

// =============================================================================
// E0211: Return Outside Function
// =============================================================================

#[test]
fn test_negative_return_outside_function_in_destructor() {
    let source_code = r#"
        class ResourceGuard {
            ~ResourceGuard() {
                return 42;
            }
        }
    "#;
    assert_compile_error(source_code, "E0211");
}

// =============================================================================
// E0212: Missing Return
// =============================================================================

#[test]
fn test_negative_missing_return_in_non_void_function() {
    let source_code = r#"
        int32 calculate_total() {
            int32 preliminary_sum = 10 + 20;
        }
    "#;
    assert_compile_error(source_code, "E0212");
}

// =============================================================================
// E0213: Return Type Mismatch
// =============================================================================

#[test]
fn test_negative_return_type_mismatch() {
    let source_code = r#"
        int32 compute_numeric_score() {
            return "string is not integer";
        }
    "#;
    assert_compile_error(source_code, "E0213");
}

// =============================================================================
// E0215: No Matching Overload
// =============================================================================

#[test]
fn test_negative_no_matching_overload_types() {
    let source_code = r#"
        class Processor {
            public void compute(int32 value) {}
            public void compute(string text) {}
        }

        void main() {
            Processor processor = new Processor();
            processor.compute(true);
        }
    "#;
    assert_compile_error(source_code, "E0215");
}

// =============================================================================
// E0216: Generic Arity Mismatch
// =============================================================================

#[test]
fn test_negative_generic_arity_mismatch() {
    let source_code = r#"
        class SingleContainer<TElement> {}

        void main() {
            SingleContainer<int32, float64> invalid_container;
        }
    "#;
    assert_compile_error(source_code, "E0216");
}

// =============================================================================
// E0217: Cannot Instantiate Abstract Type
// =============================================================================

#[test]
fn test_negative_cannot_instantiate_abstract_class() {
    let source_code = r#"
        abstract class AbstractBlueprint {}

        void main() {
            AbstractBlueprint blueprint_instance = new AbstractBlueprint();
        }
    "#;
    assert_compile_error(source_code, "E0217");
}

// =============================================================================
// E0218: Super Constructor Must Be First
// =============================================================================

#[test]
fn test_negative_super_constructor_must_be_first() {
    let source_code = r#"
        class ParentClass {}

        class DerivedClass extends ParentClass {
            DerivedClass() {
                int32 preliminary_work = 10;
                super();
            }
        }
    "#;
    assert_compile_error(source_code, "E0218");
}

// =============================================================================
// E0219: Break or Continue Outside Loop
// =============================================================================

#[test]
fn test_negative_break_outside_loop_at_function_level() {
    let source_code = r#"
        void main() {
            break;
        }
    "#;
    assert_compile_error(source_code, "E0219");
}

#[test]
fn test_negative_continue_outside_loop_at_function_level() {
    let source_code = r#"
        void main() {
            continue;
        }
    "#;
    assert_compile_error(source_code, "E0219");
}

// =============================================================================
// E0220: Invalid Operator Overload
// =============================================================================

#[test]
fn test_negative_invalid_operator_overload_ternary_parameter_count() {
    let source_code = r#"
        class Matrix {
            operator +(Matrix left_matrix, Matrix right_matrix, Matrix extra_matrix) {
                return left_matrix;
            }
        }
    "#;
    assert_compile_error(source_code, "E0220");
}

// =============================================================================
// E0221: Non-Indexable Type
// =============================================================================

#[test]
fn test_negative_non_indexable_type_integer() {
    let source_code = r#"
        void main() {
            int32 scalar_integer = 42;
            int32 element = scalar_integer[0];
        }
    "#;
    assert_compile_error(source_code, "E0221");
}

// =============================================================================
// E0226: Static Access Via Dot
// =============================================================================

#[test]
fn test_negative_static_field_access_via_dot_on_instance() {
    let source_code = r#"
        class ServiceConfig {
            public static int32 default_port = 8080;
        }

        void main() {
            ServiceConfig config_instance = new ServiceConfig();
            int32 extracted_port = config_instance.default_port;
        }
    "#;
    assert_compile_error(source_code, "E0226");
}

// =============================================================================
// E0227: Instance Access Via Scope Resolution
// =============================================================================

#[test]
fn test_negative_instance_field_access_via_scope_resolution() {
    let source_code = r#"
        class UserSession {
            public int32 session_id = 1;
        }

        void main() {
            int32 extracted_id = UserSession::session_id;
        }
    "#;
    assert_compile_error(source_code, "E0227");
}

// =============================================================================
// E0228: Invalid Super Call
// =============================================================================

#[test]
fn test_negative_invalid_super_call_in_root_class() {
    let source_code = r#"
        class StandaloneRoot {
            StandaloneRoot() : super() {}
        }
    "#;
    assert_compile_error(source_code, "E0228");
}

// =============================================================================
// E0229: Invalid Weak Modifier
// =============================================================================

#[test]
fn test_negative_invalid_weak_modifier_on_primitive_type() {
    let source_code = r#"
        class CacheEntry {
            weak int32 cached_count;
        }
    "#;
    assert_compile_error(source_code, "E0229");
}

// =============================================================================
// E0230: Circular Alias
// =============================================================================

#[test]
fn test_negative_circular_alias_definition() {
    let source_code = r#"
        alias PrimaryAlias = SecondaryAlias;
        alias SecondaryAlias = PrimaryAlias;
    "#;
    assert_compile_error(source_code, "E0230");
}

// =============================================================================
// E0231: Unresolved Alias Target
// =============================================================================

#[test]
fn test_negative_unresolved_alias_target() {
    let source_code = r#"
        alias MissingTargetAlias = NonExistentOriginalType;
    "#;
    assert_compile_error(source_code, "E0231");
}

// =============================================================================
// E0232: Cannot Resolve Import
// =============================================================================

#[test]
fn test_negative_cannot_resolve_import_statement() {
    let source_code = r#"
        import nonexistent.foreign.module;
    "#;
    assert_compile_error(source_code, "E0232");
}

// =============================================================================
// E0205: Invalid Base Class (Cannot Extend Interface)
// =============================================================================

#[test]
fn test_negative_superclass_must_be_class_cannot_extend_interface() {
    let source_code = r#"
        interface IExecutableProtocol {}
        class InvalidSubclass extends IExecutableProtocol {}
    "#;
    assert_compile_error(source_code, "E0205");
}

// =============================================================================
// E0237: Implemented Type Must Be Interface
// =============================================================================

#[test]
fn test_negative_implemented_type_must_be_interface() {
    let source_code = r#"
        class ConcreteParentClass {}
        class MisconfiguredClass implements ConcreteParentClass {}
    "#;
    assert_compile_error(source_code, "E0237");
}

// =============================================================================
// E0238: Argument Count Mismatch
// =============================================================================

#[test]
fn test_negative_argument_count_mismatch_too_few() {
    let source_code = r#"
        void format_pair(int32 first_number, int32 second_number) {}

        void main() {
            format_pair(42);
        }
    "#;
    assert_compile_error(source_code, "E0238");
}

#[test]
fn test_negative_argument_count_mismatch_too_many() {
    let source_code = r#"
        void format_single(int32 single_number) {}

        void main() {
            format_single(1, 2);
        }
    "#;
    assert_compile_error(source_code, "E0238");
}

// =============================================================================
// E0239: Argument Type Mismatch
// =============================================================================

#[test]
fn test_negative_argument_type_mismatch() {
    let source_code = r#"
        void verify_status(bool active_flag) {}

        void main() {
            verify_status("incompatible string");
        }
    "#;
    assert_compile_error(source_code, "E0239");
}

// =============================================================================
// E0240: Field Initializer Type Mismatch
// =============================================================================

#[test]
fn test_negative_field_initializer_type_mismatch() {
    let source_code = r#"
        class Configuration {
            int32 port_number = "not an integer";
        }
    "#;
    assert_compile_error(source_code, "E0240");
}

// =============================================================================
// E0241: Member Access On Non-Composite Type
// =============================================================================

#[test]
fn test_negative_member_access_on_primitive_integer() {
    let source_code = r#"
        void main() {
            int32 scalar_number = 100;
            int32 nested_value = scalar_number.length;
        }
    "#;
    assert_compile_error(source_code, "E0241");
}

// =============================================================================
// E0242: Super Constructor Argument Mismatch
// =============================================================================

#[test]
fn test_negative_super_constructor_argument_mismatch() {
    let source_code = r#"
        class ParentService {
            ParentService(int32 timeout_seconds, bool debug_mode) {}
        }

        class ChildService extends ParentService {
            ChildService() : super(30) {}
        }
    "#;
    assert_compile_error(source_code, "E0242");
}

// =============================================================================
// E0243: Main Method Signature Invalid
// =============================================================================

#[test]
fn test_negative_main_method_signature_invalid_return_type() {
    let source_code = r#"
        string main() {
            return "not void or int32";
        }
    "#;
    assert_compile_error(source_code, "E0243");
}

#[test]
fn test_negative_main_method_signature_invalid_parameters() {
    let source_code = r#"
        void main(int32 invalid_param_one, int32 invalid_param_two) {}
    "#;
    assert_compile_error(source_code, "E0243");
}

// =============================================================================
// E0244: Multiple Main Methods Found
// =============================================================================

#[test]
fn test_negative_multiple_main_methods_found() {
    let source_code = r#"
        class AppAlpha {
            public static void main() {}
        }

        class AppBeta {
            public static void main() {}
        }
    "#;
    assert_compile_error(source_code, "E0244");
}

// =============================================================================
// E0245: Duplicate Enum Variant (maps to E0202 DuplicateSymbol)
// =============================================================================

#[test]
fn test_negative_duplicate_enum_variant() {
    let source_code = r#"
        enum PriorityLevel {
            Normal,
            Critical,
            Normal
        }
    "#;
    assert_compile_error(source_code, "E0202");
}

// =============================================================================
// E0246: Enum Explicit Value Not Integer
// =============================================================================

#[test]
fn test_negative_enum_explicit_value_not_integer() {
    let source_code = r#"
        enum StatusCode {
            Ok = "200 OK"
        }
    "#;
    assert_compile_error(source_code, "E0246");
}

// =============================================================================
// E0254: Void Type Not Allowed Here
// =============================================================================

#[test]
fn test_negative_void_variable_declaration() {
    let source_code = r#"
        void main() {
            void invalid_void_variable = 1;
        }
    "#;
    assert_compile_error(source_code, "E0254");
}

// =============================================================================
// E0255: Unknown Type in Declaration
// =============================================================================

#[test]
fn test_negative_unknown_type_in_field_declaration() {
    let source_code = r#"
        class Widget {
            NonExistentTypeVariable unknown_holder;
        }
    "#;
    assert_compile_error(source_code, "E0255");
}

// =============================================================================
// E0260: This Used Outside Instance Method
// =============================================================================

#[test]
fn test_negative_this_used_in_static_method() {
    let source_code = r#"
        class StaticContextClass {
            public static void execute_static() {
                StaticContextClass self_reference = this;
            }
        }
    "#;
    assert_compile_error(source_code, "E0260");
}

#[test]
fn test_negative_this_used_outside_class() {
    let source_code = r#"
        void main() {
            int32 invalid_access = this.length;
        }
    "#;
    assert_compile_error(source_code, "E0260");
}

// =============================================================================
// E0263: Instance Method Access Via Static
// =============================================================================

#[test]
fn test_negative_instance_method_access_via_scope_resolution() {
    let source_code = r#"
        class ServiceWorker {
            public void run_task() {}
        }

        void main() {
            ServiceWorker::run_task();
        }
    "#;
    assert_compile_error(source_code, "E0263");
}

// =============================================================================
// E0264: Abstract Method in Non-Abstract Class
// =============================================================================

#[test]
fn test_negative_abstract_method_in_concrete_class_direct() {
    let source_code = r#"
        class PlainConcreteClass {
            abstract void missing_body_method();
        }
    "#;
    assert_compile_error(source_code, "E0264");
}

// =============================================================================
// E0269: Invalid Cast Type
// =============================================================================

#[test]
fn test_negative_invalid_cast_type_unrelated_classes() {
    let source_code = r#"
        class FelineAnimal {}
        class CanineAnimal {}

        void main() {
            FelineAnimal feline = new FelineAnimal();
            CanineAnimal canine = (CanineAnimal)feline;
        }
    "#;
    assert_compile_error(source_code, "E0269");
}

// =============================================================================
// E0270: Invalid Type for InstanceOf
// =============================================================================

#[test]
fn test_negative_invalid_type_for_instanceof_primitive() {
    let source_code = r#"
        void main() {
            bool check_result = 42 instanceof int32;
        }
    "#;
    assert_compile_error(source_code, "E0270");
}

// =============================================================================
// E0274: Conflicting Modifier Combination
// =============================================================================

#[test]
fn test_negative_conflicting_modifiers_abstract_and_static() {
    let source_code = r#"
        abstract class BaseService {
            abstract static void process_event();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_conflicting_modifiers_native_and_virtual() {
    let source_code = r#"
        class NativeVirtualBridge {
            native virtual void dispatch_call();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

// =============================================================================
// E0275: Modifier Not Applicable to Declaration
// =============================================================================

#[test]
fn test_negative_modifier_not_applicable_const_on_method() {
    let source_code = r#"
        class CalculationEngine {
            const void execute_step() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_modifier_not_applicable_static_on_interface_method() {
    let source_code = r#"
        interface IContract {
            static void mandatory_routine();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

// =============================================================================
// E0199: Modifier Not Valid on Package-Level Function
// =============================================================================

#[test]
fn test_negative_virtual_on_free_function() {
    let source_code = r#"
        virtual void non_method_virtual_routine() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

#[test]
fn test_negative_override_on_free_function() {
    let source_code = r#"
        override void non_method_override_routine() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

// =============================================================================
// E0278: Native on Non-Method or Function
// =============================================================================

#[test]
fn test_negative_native_on_field_declaration() {
    let source_code = r#"
        class HardwareBridge {
            native int32 raw_register;
        }
    "#;
    assert_compile_error(source_code, "E0278");
}

// =============================================================================
// E0282: Override Without Base Method
// =============================================================================

#[test]
fn test_negative_override_without_base_method() {
    let source_code = r#"
        class EmptyBase {}

        class DerivedWithOrphanOverride extends EmptyBase {
            override void non_existent_base_method() {}
        }
    "#;
    assert_compile_error(source_code, "E0282");
}

// =============================================================================
// E0283: Override Non-Virtual Method
// =============================================================================

#[test]
fn test_negative_override_non_virtual_method() {
    let source_code = r#"
        class SolidBase {
            public void non_virtual_routine() {}
        }

        class BrokenDerived extends SolidBase {
            override void non_virtual_routine() {}
        }
    "#;
    assert_compile_error(source_code, "E0283");
}

// =============================================================================
// E0286: Parameter Default Value Mismatch
// =============================================================================

#[test]
fn test_negative_parameter_default_value_type_mismatch() {
    let source_code = r#"
        void format_with_defaults(int32 timeout_limit = "not an integer") {}
    "#;
    assert_compile_error(source_code, "E0286");
}

// =============================================================================
// E0288: Bitwise Operator on Non-Integer
// =============================================================================

#[test]
fn test_negative_bitwise_operator_on_float_types() {
    let source_code = r#"
        void main() {
            float64 left_float = 3.14;
            float64 right_float = 2.71;
            float64 bitwise_result = left_float & right_float;
        }
    "#;
    assert_compile_error(source_code, "E0288");
}

#[test]
fn test_negative_bitwise_not_operator_on_float_type() {
    let source_code = r#"
        void main() {
            float64 scalar_float = 1.618;
            float64 negated_result = ~scalar_float;
        }
    "#;
    assert_compile_error(source_code, "E0288");
}
