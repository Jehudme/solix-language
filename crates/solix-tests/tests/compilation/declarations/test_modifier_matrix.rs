//! Conformance test suite for the 13-Modifier Legality Matrix across all declaration targets.

#![forbid(unsafe_code)]

use solix_tests::common::assert_compile_error;

// =============================================================================
// 1. Constructor Modifier Legality Tests (Allowed: public, protected, private, internal, inline)
// =============================================================================

#[test]
fn test_negative_constructor_static_modifier() {
    let source_code = r#"
        class Widget {
            static Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_constructor_virtual_modifier() {
    let source_code = r#"
        class Widget {
            virtual Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_constructor_override_modifier() {
    let source_code = r#"
        class Widget {
            override Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_constructor_abstract_modifier() {
    let source_code = r#"
        abstract class Widget {
            abstract Widget();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_constructor_const_modifier() {
    let source_code = r#"
        class Widget {
            const Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_constructor_weak_modifier() {
    let source_code = r#"
        class Widget {
            weak Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

// =============================================================================
// 2. Field Modifier Legality Tests (Allowed: public, protected, private, internal, static, const, weak, extern)
// =============================================================================

#[test]
fn test_negative_field_virtual_modifier() {
    let source_code = r#"
        class Widget {
            virtual int32 value = 1;
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_field_override_modifier() {
    let source_code = r#"
        class Widget {
            override int32 value = 1;
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_field_abstract_modifier() {
    let source_code = r#"
        class Widget {
            abstract int32 value;
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_field_inline_modifier() {
    let source_code = r#"
        class Widget {
            inline int32 value = 1;
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_field_native_modifier() {
    let source_code = r#"
        class Widget {
            native int32 value;
        }
    "#;
    assert_compile_error(source_code, "E0278");
}

#[test]
fn test_negative_field_weak_on_primitive_type() {
    let source_code = r#"
        class Widget {
            weak int32 count;
        }
    "#;
    assert_compile_error(source_code, "E0229");
}

#[test]
fn test_negative_field_weak_on_boolean_type() {
    let source_code = r#"
        class Widget {
            weak bool flag;
        }
    "#;
    assert_compile_error(source_code, "E0229");
}

#[test]
fn test_negative_field_weak_on_float_type() {
    let source_code = r#"
        class Widget {
            weak float64 weight;
        }
    "#;
    assert_compile_error(source_code, "E0229");
}

// =============================================================================
// 3. Method & Free Function Modifier Legality Tests
// =============================================================================

#[test]
fn test_negative_method_const_modifier() {
    let source_code = r#"
        class Widget {
            const void compute() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_method_weak_modifier() {
    let source_code = r#"
        class Widget {
            weak void compute() {}
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_free_function_const_modifier() {
    let source_code = r#"
        const void standalone_calculator() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

#[test]
fn test_negative_free_function_weak_modifier() {
    let source_code = r#"
        weak void standalone_calculator() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

#[test]
fn test_negative_free_function_protected_modifier() {
    let source_code = r#"
        protected void standalone_calculator() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

#[test]
fn test_negative_free_function_private_modifier() {
    let source_code = r#"
        private void standalone_calculator() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

// =============================================================================
// 4. Destructor Modifier Legality Tests (Destructors allow NO modifiers)
// =============================================================================

#[test]
fn test_negative_destructor_public_modifier() {
    let source_code = r#"
        class Widget {
            public ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_private_modifier() {
    let source_code = r#"
        class Widget {
            private ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_protected_modifier() {
    let source_code = r#"
        class Widget {
            protected ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_internal_modifier() {
    let source_code = r#"
        class Widget {
            internal ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_static_modifier() {
    let source_code = r#"
        class Widget {
            static ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_virtual_modifier() {
    let source_code = r#"
        class Widget {
            virtual ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_override_modifier() {
    let source_code = r#"
        class Widget {
            override ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_abstract_modifier() {
    let source_code = r#"
        abstract class Widget {
            abstract ~Widget();
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_inline_modifier() {
    let source_code = r#"
        class Widget {
            inline ~Widget() {}
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_native_modifier() {
    let source_code = r#"
        class Widget {
            native ~Widget();
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_destructor_extern_modifier() {
    let source_code = r#"
        class Widget {
            extern ~Widget();
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

// =============================================================================
// 5. Interface Member Modifier Legality Tests (Allowed: NONE or public only)
// =============================================================================

#[test]
fn test_negative_interface_member_private() {
    let source_code = r#"
        interface IService {
            private void execute();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_interface_member_protected() {
    let source_code = r#"
        interface IService {
            protected void execute();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_interface_member_static() {
    let source_code = r#"
        interface IService {
            static void execute();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_interface_member_native() {
    let source_code = r#"
        interface IService {
            native void execute();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_interface_member_inline() {
    let source_code = r#"
        interface IService {
            inline void execute();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_interface_member_override() {
    let source_code = r#"
        interface IService {
            override void execute();
        }
    "#;
    assert_compile_error(source_code, "E0275");
}

#[test]
fn test_negative_interface_method_with_body() {
    let source_code = r#"
        interface IService {
            void execute() {
                int32 counter = 1;
            }
        }
    "#;
    assert_compile_error(source_code, "E0127");
}

// =============================================================================
// 6. Native & Abstract Modifier Conflict Tests
// =============================================================================

#[test]
fn test_negative_native_combined_with_abstract() {
    let source_code = r#"
        abstract class Widget {
            native abstract void render();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_native_combined_with_virtual() {
    let source_code = r#"
        class Widget {
            native virtual void render();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_native_combined_with_override() {
    let source_code = r#"
        class Base {
            public virtual void render() {}
        }
        class Widget extends Base {
            native override void render();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_native_combined_with_inline() {
    let source_code = r#"
        class Widget {
            native inline void render();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_native_combined_with_extern() {
    let source_code = r#"
        class Widget {
            native extern void render();
        }
    "#;
    assert_compile_error(source_code, "E0156");
}

#[test]
fn test_negative_generic_native_function() {
    let source_code = r#"
        class Widget {
            native void process<T>(T item);
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_abstract_combined_with_static() {
    let source_code = r#"
        abstract class Widget {
            abstract static void render();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_abstract_combined_with_private() {
    let source_code = r#"
        abstract class Widget {
            abstract private void render();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_abstract_combined_with_inline() {
    let source_code = r#"
        abstract class Widget {
            abstract inline void render();
        }
    "#;
    assert_compile_error(source_code, "E0274");
}

#[test]
fn test_negative_abstract_combined_with_extern() {
    let source_code = r#"
        abstract class Widget {
            abstract extern void render();
        }
    "#;
    assert_compile_error(source_code, "E0156");
}

#[test]
fn test_negative_abstract_method_in_concrete_class() {
    let source_code = r#"
        class ConcreteWidget {
            abstract void draw();
        }
    "#;
    assert_compile_error(source_code, "E0264");
}

#[test]
fn test_negative_abstract_method_with_body() {
    let source_code = r#"
        abstract class BaseWidget {
            abstract void draw() {
                int32 number = 42;
            }
        }
    "#;
    assert_compile_error(source_code, "E0138");
}

#[test]
fn test_negative_native_method_with_body() {
    let source_code = r#"
        class BaseWidget {
            native void draw() {
                int32 number = 42;
            }
        }
    "#;
    assert_compile_error(source_code, "E0138");
}

// =============================================================================
// 7. Extern Modifier Legality Tests (E0137, E0154, E0155, E0156, E0277)
// =============================================================================

#[test]
fn test_negative_extern_variable_with_initializer() {
    let source_code = r#"
        extern int32 external_counter = 100;
    "#;
    assert_compile_error(source_code, "E0154");
}

#[test]
fn test_negative_extern_class_field_with_initializer() {
    let source_code = r#"
        class Widget {
            extern int32 external_field = 42;
        }
    "#;
    assert_compile_error(source_code, "E0154");
}

#[test]
fn test_negative_extern_inside_local_function() {
    let source_code = r#"
        void test() {
            extern int32 local_extern;
        }
    "#;
    assert_compile_error(source_code, "E0155");
}

#[test]
fn test_negative_extern_combined_with_inline() {
    let source_code = r#"
        extern inline void fast_routine();
    "#;
    assert_compile_error(source_code, "E0156");
}

#[test]
fn test_negative_extern_combined_with_abstract() {
    let source_code = r#"
        extern abstract void pending_routine();
    "#;
    assert_compile_error(source_code, "E0156");
}

#[test]
fn test_negative_extern_function_with_body() {
    let source_code = r#"
        extern void implemented_routine() {
            int32 counter = 1;
        }
    "#;
    assert_compile_error(source_code, "E0137");
}

#[test]
fn test_negative_extern_method_with_body() {
    let source_code = r#"
        class Widget {
            extern void implemented_method() {
                int32 counter = 1;
            }
        }
    "#;
    assert_compile_error(source_code, "E0137");
}

#[test]
fn test_negative_extern_constructor_with_body() {
    let source_code = r#"
        class Widget {
            extern Widget() {
                int32 counter = 1;
            }
        }
    "#;
    assert_compile_error(source_code, "E0137");
}

#[test]
fn test_negative_extern_on_enum() {
    let source_code = r#"
        extern enum BadEnum { A, B }
    "#;
    assert_compile_error(source_code, "E0198");
}

#[test]
fn test_negative_extern_on_destructor() {
    let source_code = r#"
        class Widget {
            extern ~Widget();
        }
    "#;
    assert_compile_error(source_code, "E0130");
}

#[test]
fn test_negative_extern_on_operator() {
    let source_code = r#"
        class Widget {
            extern operator +(Widget left, Widget right);
        }
    "#;
    assert_compile_error(source_code, "E0200");
}
