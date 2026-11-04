//! LiteralNode Conformance Tests (SPEC-0001 §14, TESTS.md Section 4.9, Suite 52).

#![forbid(unsafe_code)]

use solix_runtime::value::model::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_all_literal_types() {
    let code = r#"
        int32 main() {
            int64 big = 10000000000L;
            char letter = 'Z';
            bool flag = true;
            float64 pi = 3.14;
            if (letter != 'Z' || !flag) return 1;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_hexadecimal_and_binary_numeric_literals() {
    let code = r#"
        int32 main() {
            int32 hex = 0x2A;
            int32 bin = 0b101010;
            return (hex == 42 && bin == 42) ? 0 : 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_character_literals_with_escapes() {
    let code = r#"
        int32 main() {
            char newline = '\n';
            char tab = '\t';
            char quote = '\'';
            if (newline == 10 && tab == 9 && quote == 39) {
                return 0;
            }
            return 1;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_integer_literal_overflow() {
    // E0150: Integer literal out of range
    let code = r#"
        void test() {
            int32 x = 99999999999999999999;
        }
    "#;
    assert_compile_error(code, "exceeds");
}

#[test]
fn test_case_4_2_unterminated_string_literal() {
    // E0101: Unterminated string literal
    let code = r#"
        void test() {
            char[] s = "unterminated;
        }
    "#;
    assert_compile_error(code, "E0011");
}

#[test]
fn test_case_52_1_common_whitespace_and_punctuation_escapes() {
    let code = r#"
        int32 main() {
            char[] s = "hello\nworld\ttab\\slash\"quote";
            if (s[5] != (char)10) return 1;
            if (s[11] != (char)9) return 2;
            if (s[15] != (char)92) return 3;
            if (s[21] != (char)34) return 4;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_52_2_hexadecimal_escape_sequences_and_null_character() {
    let code = r#"
        int32 main() {
            char[] s = "A\x42C\0D";
            if (s[0] != 'A') return 1;
            if (s[1] != 'B') return 2;
            if (s[2] != 'C') return 3;
            if (s[3] != '\0') return 4;
            if (s[4] != 'D') return 5;
            return 0;
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_52_3_unterminated_hex_escape_in_string_literal() {
    // E0151: Invalid escape sequence
    let code = r#"
        void test() {
            char[] s = "bad\x4";
        }
    "#;
    assert_compile_error(code, "escape");
}
