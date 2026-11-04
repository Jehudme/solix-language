//! Suite: FieldDeclaration - Declarations (TESTS.md)

use solix_runtime::{RuntimeErrorCode, Value};
use solix_tests::common::{assert_compile_error, assert_runtime_error, compile_and_run};

#[test]
fn test_case_3_1_cycle_breaking_with_weak_references() {
    let code = r#"
        class Parent {
            public Child c;
        }
        class Child {
            public weak Parent p;
        }

        class Main {
            public static int32 main() {
                Parent p = new Parent();
                Child c = new Child();
                p.c = c;
                c.p = p;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_field_access_modifiers() {
    let code = r#"
        class Account {
            public int32 id;
            private int32 secret;
            protected int32 balance;

            public Account(int32 id, int32 sec, int32 bal) {
                this.id = id;
                this.secret = sec;
                this.balance = bal;
            }
            public int32 getSecret() { return this.secret; }
        }

        class Main {
            public static int32 main() {
                Account a = new Account(1, 1234, 500);
                return a.getSecret() == 1234 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_static_class_fields() {
    let code = r#"
        class Counter {
            public static int32 count;
        }

        class Main {
            public static int32 main() {
                Counter::count = 10;
                Counter c = new Counter();
                Counter::count++;
                return Counter::count == 11 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_constant_field_declaration() {
    let code = r#"
        class Config {
            public const int32 MAX_USERS = 500;
        }

        class Main {
            public static int32 main() {
                return Config::MAX_USERS == 500 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_5_in_class_field_initialization() {
    let code = r#"
        class Item {
            public int32 id;
            public Item(int32 i) { this.id = i; }
        }

        class Entity {
            private Item item = new Item(42);
            private int32 count = 100;

            public int32 getItemId() {
                return this.item.id;
            }
            public int32 getCount() {
                return this.count;
            }
        }

        class Main {
            public static int32 main() {
                Entity e = new Entity();
                return (e.getItemId() == 42 && e.getCount() == 100) ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_6_static_field_initialization_with_object() {
    let code = r#"
        class Tag {
            public int32 code;
            public Tag(int32 c) { this.code = c; }
        }

        class Config {
            public static Tag tag = new Tag(99);
            public static int32 version = 3;

            public static Tag getTag() {
                return Config::tag;
            }
        }

        class Main {
            public static int32 main() {
                return (Config::getTag().code == 99 && Config::version == 3) ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_7_deterministic_live_object_count_reclamation_on_scope_exit() {
    let code = r#"
        class Parent {
            public Child c;
        }
        class Child {
            public weak Parent p;
        }

        class Main {
            public static int32 main() {
                Parent p = new Parent();
                Child c = new Child();
                p.c = c;
                c.p = p;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_8_contiguous_field_packing_and_alignment() {
    let code = r#"
        class PackedStruct {
            public uint8 b1;
            public uint8 b2;
            public uint16 s1;
            public uint32 i1;
            public uint64 l1;

            public PackedStruct(uint8 b1, uint8 b2, uint16 s1, uint32 i1, uint64 l1) {
                this.b1 = b1;
                this.b2 = b2;
                this.s1 = s1;
                this.i1 = i1;
                this.l1 = l1;
            }
        }

        class Main {
            public static int32 main() {
                PackedStruct p = new PackedStruct(10, 20, 300, 40000, 5000000000);
                if (p.b1 != 10) return 1;
                if (p.b2 != 20) return 2;
                if (p.s1 != 300) return 3;
                if (p.i1 != 40000) return 4;
                if (p.l1 != 5000000000) return 5;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_9_narrow_signed_and_unsigned_fields() {
    let code = r#"
        class SignExtensionTest {
            public int8 s8;
            public uint8 u8;
            public int16 s16;
            public uint16 u16;
            public int32 s32;
            public uint32 u32;

            public SignExtensionTest(int8 a, uint8 b, int16 c, uint16 d, int32 e, uint32 f) {
                this.s8 = a;
                this.u8 = b;
                this.s16 = c;
                this.u16 = d;
                this.s32 = e;
                this.u32 = f;
            }
        }

        class Main {
            public static int32 main() {
                SignExtensionTest t = new SignExtensionTest(-5, 250, -300, 65000, -70000, 3000000000);
                if (t.s8 != -5) return 1;
                if (t.u8 != 250) return 2;
                if (t.s16 != -300) return 3;
                if (t.u16 != 65000) return 4;
                if (t.s32 != -70000) return 5;
                if (t.u32 != 3000000000) return 6;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_10_segregated_free_lists_and_adjacent_block_coalescing() {
    let code = r#"
        class Node {
            public int32 val;
            public Node(int32 v) { this.val = v; }
        }

        class Main {
            public static int32 main() {
                for (int32 i = 0; i < 100; i++) {
                    Node n1 = new Node(i);
                    Node n2 = new Node(i + 1);
                    Node n3 = new Node(i + 2);
                }
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_duplicate_field_identifier() {
    // Error: E0202 (DuplicateSymbol)
    let code = r#"
        class Item {
            int32 count;
            float64 count;
        }
    "#;
    assert_compile_error(code, "E0202");
}

#[test]
fn test_case_4_2_field_access_on_null_reference_runtime_fault() {
    let code = r#"
        class Item {
            public int32 count;
        }

        class Main {
            public static void main() {
                Item item = null;
                item.count = 5;
            }
        }
    "#;
    assert_runtime_error(code, RuntimeErrorCode::NullReferenceAccess);
}

#[test]
fn test_case_4_3_accessing_private_field_outside_class() {
    // Error: E0208 (PrivateAccessViolation)
    let code = r#"
        class Vault {
            private int32 passcode;
        }

        class Main {
            public static void test() {
                Vault v = new Vault();
                int32 x = v.passcode;
            }
        }
    "#;
    assert_compile_error(code, "E0208");
}

#[test]
fn test_case_4_4_modifying_const_field() {
    // Error: E0210 (CannotAssignToConst)
    let code = r#"
        class Constants {
            public const int32 RATE = 5;
        }

        class Main {
            public static void test() {
                Constants::RATE = 10;
            }
        }
    "#;
    assert_compile_error(code, "E0210");
}

#[test]
fn test_case_4_5_incompatible_field_initializer_type() {
    // Error: E0240 (FieldInitializerTypeMismatch)
    let code = r#"
        class Model {
            public int32 count = "invalid";
        }
    "#;
    assert_compile_error(code, "E0240");
}

#[test]
fn test_case_4_7_field_access_modifier_on_local_variable() {
    // Error: E0117 (ModifierOnLocalVariable)
    let code = r#"
        class Main {
            public static void test() {
                public int32 x = 10;
            }
        }
    "#;
    assert_compile_error(code, "E0117");
}

#[test]
fn test_case_4_8_standalone_field_declared_at_file_top_level() {
    // Error: E0175 (StandaloneFieldWithAccessModifier)
    let code = r#"
        public int32 globalField = 42;
    "#;
    assert_compile_error(code, "E0175");
}
