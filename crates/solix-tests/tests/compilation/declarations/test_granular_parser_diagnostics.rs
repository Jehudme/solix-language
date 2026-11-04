//! Conformance test suite for granular parser and lexer diagnostics (E0011-E0015, E0160-E0200).

#![forbid(unsafe_code)]

use solix_tests::common::assert_compile_error;

// =============================================================================
// 1. Lexer Diagnostic Error Codes (E0011 - E0015)
// =============================================================================

#[test]
fn test_negative_e0011_unterminated_string_at_eof() {
    let source_code = r#"
        void test() {
            string text = "unterminated string at end of file;
        }
    "#;
    assert_compile_error(source_code, "E0011");
}

#[test]
fn test_negative_e0012_unterminated_character_literal() {
    let source_code = r#"
        void test() {
            char symbol = 'a;
        }
    "#;
    assert_compile_error(source_code, "E0012");
}

#[test]
fn test_negative_e0013_unterminated_block_comment() {
    let source_code = r#"
        /* This is an unclosed block comment
        void test() {}
    "#;
    assert_compile_error(source_code, "E0013");
}

#[test]
fn test_negative_e0014_hex_prefix_missing_digits() {
    let source_code = r#"
        void test() {
            int32 number = 0x;
        }
    "#;
    assert_compile_error(source_code, "E0014");
}

#[test]
fn test_negative_e0015_binary_prefix_missing_digits() {
    let source_code = r#"
        void test() {
            int32 number = 0b;
        }
    "#;
    assert_compile_error(source_code, "E0015");
}

// =============================================================================
// 2. Specific Delimiter Error Codes (E0160 - E0171) & Header Errors (E0141 - E0144)
// =============================================================================

#[test]
fn test_negative_e0160_expected_open_paren_in_sizeof() {
    let source_code = r#"
        void test() {
            int32 size = sizeof int32);
        }
    "#;
    assert_compile_error(source_code, "E0160");
}

#[test]
fn test_negative_e0160_expected_open_paren_in_default() {
    let source_code = r#"
        void test() {
            int32 def = default int32);
        }
    "#;
    assert_compile_error(source_code, "E0160");
}

#[test]
fn test_negative_e0160_expected_open_paren_in_tuple() {
    let source_code = r#"
        void test() {
            tuple 1, 2);
        }
    "#;
    assert_compile_error(source_code, "E0160");
}

#[test]
fn test_negative_e0160_expected_open_paren_in_assert() {
    let source_code = r#"
        void test() {
            assert 1 == 1;
        }
    "#;
    assert_compile_error(source_code, "E0160");
}

#[test]
fn test_negative_e0160_expected_open_paren_in_exit() {
    let source_code = r#"
        void test() {
            exit 0;
        }
    "#;
    assert_compile_error(source_code, "E0160");
}

#[test]
fn test_negative_e0160_expected_open_paren_in_if() {
    let source_code = r#"
        void test() {
            if true) {}
        }
    "#;
    assert_compile_error(source_code, "E0160");
}

#[test]
fn test_negative_e0141_malformed_catch_parameter() {
    let source_code = r#"
        class Exc {}
        void test() {
            try {} catch Exc error) {}
        }
    "#;
    assert_compile_error(source_code, "E0141");
}

#[test]
fn test_negative_e0142_malformed_switch() {
    let source_code = r#"
        void test() {
            switch 42) {
                default: break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0142");
}

#[test]
fn test_negative_e0143_malformed_for_loop() {
    let source_code = r#"
        void test() {
            for int32 idx = 0; idx < 10; idx = idx + 1) {}
        }
    "#;
    assert_compile_error(source_code, "E0143");
}

#[test]
fn test_negative_e0144_malformed_while_condition() {
    let source_code = r#"
        void test() {
            while true) {}
        }
    "#;
    assert_compile_error(source_code, "E0144");
}

#[test]
fn test_negative_e0161_expected_close_brace_in_class() {
    let source_code = r#"
        class IncompleteClass {
            int32 field;
    "#;
    assert_compile_error(source_code, "E0161");
}

#[test]
fn test_negative_e0161_expected_close_brace_in_method() {
    let source_code = r#"
        class Sample {
            void run() {
                int32 counter = 1;
        }
    "#;
    assert_compile_error(source_code, "E0161");
}

#[test]
fn test_negative_e0161_expected_close_brace_in_block() {
    let source_code = r#"
        void test() {
            {
                int32 inner = 10;
        }
    "#;
    assert_compile_error(source_code, "E0161");
}

#[test]
fn test_negative_e0162_expected_open_brace_in_class() {
    let source_code = r#"
        class MissingBraceClass
            int32 field;
        }
    "#;
    assert_compile_error(source_code, "E0162");
}

#[test]
fn test_negative_e0162_expected_open_brace_in_interface() {
    let source_code = r#"
        interface MissingBraceInterface
            void execute();
        }
    "#;
    assert_compile_error(source_code, "E0162");
}

#[test]
fn test_negative_e0163_expected_close_bracket_in_array_access() {
    let source_code = r#"
        void test() {
            int32[] items = new int32[5];
            int32 first = items[0;
        }
    "#;
    assert_compile_error(source_code, "E0163");
}

#[test]
fn test_negative_e0163_expected_close_bracket_in_array_creation() {
    let source_code = r#"
        void test() {
            int32[] numbers = new int32[10;
        }
    "#;
    assert_compile_error(source_code, "E0163");
}

#[test]
fn test_negative_e0165_expected_close_angle_in_generic_parameter() {
    let source_code = r#"
        alias Container<T = int32;
    "#;
    assert_compile_error(source_code, "E0165");
}

#[test]
fn test_negative_e0166_expected_colon_in_ternary() {
    let source_code = r#"
        void test() {
            int32 result = true ? 1 2;
        }
    "#;
    assert_compile_error(source_code, "E0166");
}

#[test]
fn test_negative_e0167_expected_colon_in_switch_case() {
    let source_code = r#"
        void test() {
            switch (1) {
                case 1
                    break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0167");
}

#[test]
fn test_negative_e0167_expected_colon_in_switch_default() {
    let source_code = r#"
        void test() {
            switch (1) {
                default
                    break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0167");
}

#[test]
fn test_negative_e0168_expected_colon_after_super() {
    let source_code = r#"
        class Base {
            public Base(int32 number) {}
        }
        class Derived extends Base {
            public Derived() super(10) {}
        }
    "#;
    assert_compile_error(source_code, "E0168");
}

#[test]
fn test_negative_e0169_expected_comma_in_cast() {
    let source_code = r#"
        void test() {
            int32 value = cast(42 int32);
        }
    "#;
    assert_compile_error(source_code, "E0169");
}

#[test]
fn test_negative_e0170_expected_constructor_call_or_size_after_new() {
    let source_code = r#"
        class Widget {}
        void test() {
            Widget item = new Widget;
        }
    "#;
    assert_compile_error(source_code, "E0170");
}

#[test]
fn test_negative_e0171_unexpected_question_operator() {
    let source_code = r#"
        void test() {
            int32 value = ?;
        }
    "#;
    assert_compile_error(source_code, "E0171");
}

// =============================================================================
// 3. Parameter, Header & Construct Error Codes (E0176 - E0195)
// =============================================================================

#[test]
fn test_negative_e0176_malformed_function_pointer_type() {
    let source_code = r#"
        void test() {
            int32(*) int32 callback;
        }
    "#;
    assert_compile_error(source_code, "E0176");
}

#[test]
fn test_negative_e0177_expected_catch_or_finally_block() {
    let source_code = r#"
        class Exc {}
        void test() {
            try {} catch (Exc error) int32 x = 1;
        }
    "#;
    assert_compile_error(source_code, "E0177");
}

#[test]
fn test_negative_e0178_expected_identifier_in_parameter() {
    let source_code = r#"
        void process(int32) {}
    "#;
    assert_compile_error(source_code, "E0178");
}

#[test]
fn test_negative_e0179_expected_parameter_list() {
    let source_code = r#"
        class Service {
            void process {
            }
        }
    "#;
    assert_compile_error(source_code, "E0179");
}

#[test]
fn test_negative_e0180_expected_interface_list_after_implements() {
    let source_code = r#"
        class BadImplements implements {
        }
    "#;
    assert_compile_error(source_code, "E0180");
}

#[test]
fn test_negative_e0181_expected_superclass_list_after_extends() {
    let source_code = r#"
        class BadExtends extends {
        }
    "#;
    assert_compile_error(source_code, "E0181");
}

#[test]
fn test_negative_e0182_expected_enum_brace() {
    let source_code = r#"
        enum BadEnum RED, GREEN, BLUE }
    "#;
    assert_compile_error(source_code, "E0182");
}

#[test]
fn test_negative_e0183_expected_switch_brace() {
    let source_code = r#"
        void test() {
            switch (1)
                case 1: break;
        }
    "#;
    assert_compile_error(source_code, "E0183");
}

#[test]
fn test_negative_e0184_duplicate_default_in_switch() {
    let source_code = r#"
        void test() {
            switch (1) {
                default: break;
                default: break;
            }
        }
    "#;
    assert_compile_error(source_code, "E0184");
}

#[test]
fn test_negative_e0186_expected_expression_in_index() {
    let source_code = r#"
        void test() {
            int32[] items = new int32[5];
            int32 first = items[];
        }
    "#;
    assert_compile_error(source_code, "E0186");
}

#[test]
fn test_negative_e0187_empty_tuple_expression() {
    let source_code = r#"
        void test() {
            tuple();
        }
    "#;
    assert_compile_error(source_code, "E0187");
}

#[test]
fn test_negative_e0188_malformed_type_bound() {
    let source_code = r#"
        class GenericContainer<T : > {}
    "#;
    assert_compile_error(source_code, "E0188");
}

#[test]
fn test_negative_e0189_expected_condition_in_if() {
    let source_code = r#"
        void test() {
            if () {}
        }
    "#;
    assert_compile_error(source_code, "E0189");
}

#[test]
fn test_negative_e0190_expected_condition_in_while() {
    let source_code = r#"
        void test() {
            while () {}
        }
    "#;
    assert_compile_error(source_code, "E0190");
}

#[test]
fn test_negative_e0191_empty_enum_declaration() {
    let source_code = r#"
        enum EmptyEnum {}
    "#;
    assert_compile_error(source_code, "E0191");
}

#[test]
fn test_negative_e0192_unexpected_semicolon() {
    let source_code = r#"
        class ValidClass {
            void run() {
                ;
            }
        }
    "#;
    assert_compile_error(source_code, "E0192");
}

#[test]
fn test_negative_e0193_missing_operator_in_expression() {
    let source_code = r#"
        void test() {
            int32 number = 10 20;
        }
    "#;
    assert_compile_error(source_code, "E0193");
}

#[test]
fn test_negative_e0194_malformed_constructor_chaining() {
    let source_code = r#"
        class Base {}
        class Derived extends Base {
            public Derived() : this.bad {}
        }
    "#;
    assert_compile_error(source_code, "E0194");
}

#[test]
fn test_negative_e0195_invalid_top_level_token() {
    let source_code = r#"
        catch (Exc e) {}
    "#;
    assert_compile_error(source_code, "E0195");
}

// =============================================================================
// 4. Parser Modifier Legality Matrix Diagnostic Codes (E0172 - E0200)
// =============================================================================

#[test]
fn test_negative_e0172_duplicate_public_modifier() {
    let source_code = r#"
        public public class Widget {}
    "#;
    assert_compile_error(source_code, "E0172");
}

#[test]
fn test_negative_e0172_duplicate_static_modifier() {
    let source_code = r#"
        public static static void run() {}
    "#;
    assert_compile_error(source_code, "E0172");
}

#[test]
fn test_negative_e0172_duplicate_const_modifier() {
    let source_code = r#"
        class Widget {
            const const int32 value = 42;
        }
    "#;
    assert_compile_error(source_code, "E0172");
}

#[test]
fn test_negative_e0173_conflicting_public_and_private() {
    let source_code = r#"
        public private class Widget {}
    "#;
    assert_compile_error(source_code, "E0173");
}

#[test]
fn test_negative_e0173_conflicting_public_and_protected() {
    let source_code = r#"
        public protected class Widget {}
    "#;
    assert_compile_error(source_code, "E0173");
}

#[test]
fn test_negative_e0173_conflicting_protected_and_private() {
    let source_code = r#"
        protected private class Widget {}
    "#;
    assert_compile_error(source_code, "E0173");
}

#[test]
fn test_negative_e0173_conflicting_internal_and_private() {
    let source_code = r#"
        internal private class Widget {}
    "#;
    assert_compile_error(source_code, "E0173");
}

#[test]
fn test_negative_e0174_static_on_free_function() {
    let source_code = r#"
        static void standalone_function() {}
    "#;
    assert_compile_error(source_code, "E0174");
}

#[test]
fn test_negative_e0175_standalone_field_with_public() {
    let source_code = r#"
        public int32 global_variable = 42;
    "#;
    assert_compile_error(source_code, "E0175");
}

#[test]
fn test_negative_e0175_standalone_field_with_private() {
    let source_code = r#"
        private int32 global_variable = 42;
    "#;
    assert_compile_error(source_code, "E0175");
}

#[test]
fn test_negative_e0175_standalone_field_with_protected() {
    let source_code = r#"
        protected int32 global_variable = 42;
    "#;
    assert_compile_error(source_code, "E0175");
}

#[test]
fn test_negative_e0175_standalone_field_with_internal() {
    let source_code = r#"
        internal int32 global_variable = 42;
    "#;
    assert_compile_error(source_code, "E0175");
}

#[test]
fn test_negative_e0185_modifier_on_package_statement() {
    let source_code = r#"
        public package my.pkg;
    "#;
    assert_compile_error(source_code, "E0185");
}

#[test]
fn test_negative_e0185_modifier_on_import_statement() {
    let source_code = r#"
        package my.pkg;
        public import other.pkg.Item;
    "#;
    assert_compile_error(source_code, "E0185");
}

#[test]
fn test_negative_e0185_modifier_on_alias_statement() {
    let source_code = r#"
        package my.pkg;
        public alias Number = int32;
    "#;
    assert_compile_error(source_code, "E0185");
}

#[test]
fn test_negative_e0196_invalid_class_modifier_virtual() {
    let source_code = r#"
        virtual class BadClass {}
    "#;
    assert_compile_error(source_code, "E0196");
}

#[test]
fn test_negative_e0196_invalid_class_modifier_override() {
    let source_code = r#"
        override class BadClass {}
    "#;
    assert_compile_error(source_code, "E0196");
}

#[test]
fn test_negative_e0196_invalid_class_modifier_static() {
    let source_code = r#"
        static class BadClass {}
    "#;
    assert_compile_error(source_code, "E0196");
}

#[test]
fn test_negative_e0196_invalid_class_modifier_inline() {
    let source_code = r#"
        inline class BadClass {}
    "#;
    assert_compile_error(source_code, "E0196");
}

#[test]
fn test_negative_e0197_invalid_interface_modifier_virtual() {
    let source_code = r#"
        virtual interface BadInterface {}
    "#;
    assert_compile_error(source_code, "E0197");
}

#[test]
fn test_negative_e0197_invalid_interface_modifier_static() {
    let source_code = r#"
        static interface BadInterface {}
    "#;
    assert_compile_error(source_code, "E0197");
}

#[test]
fn test_negative_e0197_invalid_interface_modifier_abstract() {
    let source_code = r#"
        abstract interface BadInterface {}
    "#;
    assert_compile_error(source_code, "E0197");
}

#[test]
fn test_negative_e0198_invalid_enum_modifier_abstract() {
    let source_code = r#"
        abstract enum BadEnum { A, B }
    "#;
    assert_compile_error(source_code, "E0198");
}

#[test]
fn test_negative_e0198_invalid_enum_modifier_virtual() {
    let source_code = r#"
        virtual enum BadEnum { A, B }
    "#;
    assert_compile_error(source_code, "E0198");
}

#[test]
fn test_negative_e0198_invalid_enum_modifier_static() {
    let source_code = r#"
        static enum BadEnum { A, B }
    "#;
    assert_compile_error(source_code, "E0198");
}

#[test]
fn test_negative_e0199_invalid_free_function_modifier_virtual() {
    let source_code = r#"
        virtual void free_func() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

#[test]
fn test_negative_e0199_invalid_free_function_modifier_override() {
    let source_code = r#"
        override void free_func() {}
    "#;
    assert_compile_error(source_code, "E0199");
}

#[test]
fn test_negative_e0199_invalid_free_function_modifier_abstract() {
    let source_code = r#"
        abstract void free_func();
    "#;
    assert_compile_error(source_code, "E0199");
}

#[test]
fn test_negative_e0200_invalid_operator_modifier_static() {
    let source_code = r#"
        class Complex {
            static operator +(Complex left, Complex right) { return left; }
        }
    "#;
    assert_compile_error(source_code, "E0200");
}

#[test]
fn test_negative_e0200_invalid_operator_modifier_virtual() {
    let source_code = r#"
        class Complex {
            virtual operator +(Complex left, Complex right) { return left; }
        }
    "#;
    assert_compile_error(source_code, "E0200");
}

#[test]
fn test_negative_e0200_invalid_operator_modifier_abstract() {
    let source_code = r#"
        class Complex {
            abstract operator +(Complex left, Complex right);
        }
    "#;
    assert_compile_error(source_code, "E0200");
}
