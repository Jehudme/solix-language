//! Suite: GenericsAndTemplates - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_48_1_single_parameter_generic_class() {
    let code = r#"
        public class Box<T> {
            public T val;
            public Box(T v) { this.val = v; }
            public T get() { return this.val; }
            public void set(T v) { this.val = v; }
        }

        public class Main {
            public static int32 main() {
                Box<int32> ib = new Box<int32>(42);
                ib.set(100);
                return ib.get() == 100 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_48_2_multi_parameter_generic_class() {
    let code = r#"
        public class Pair<K, V> {
            public K key;
            public V val;
            public Pair(K k, V v) {
                this.key = k;
                this.val = v;
            }
        }

        public class Main {
            public static int32 main() {
                Pair<int32, bool> p = new Pair<int32, bool>(10, true);
                if (p.key == 10 && p.val == true) {
                    return 0;
                }
                return 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_48_3_generic_class_implementing_interface() {
    let code = r#"
        public interface IContainer {
            int32 getVal();
        }

        public class Holder<T> implements IContainer {
            public T item;
            public Holder(T item) { this.item = item; }
            public int32 getVal() { return 99; }
        }

        public class Main {
            public static int32 main() {
                IContainer container = new Holder<int32>(42);
                return container.getVal() == 99 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_48_7_nested_generic_types() {
    let code = r#"
        public class Cell<T> {
            public T content;
            public Cell(T c) { this.content = c; }
        }

        public class Main {
            public static int32 main() {
                Cell<Cell<int32>> nested = new Cell<Cell<int32>>(new Cell<int32>(777));
                return nested.content.content == 777 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_48_8_generic_class_with_lambda_field() {
    let code = r#"
        public class Processor<T> {
            public T(*)(T) transform;
            public Processor(T(*)(T) fn) {
                this.transform = fn;
            }
            public T execute(T input) {
                return this.transform(input);
            }
        }

        public class Main {
            public static int32 main() {
                Processor<int32> p = new Processor<int32>((int32 x) => x * 2);
                return p.execute(21) == 42 ? 0 : 1;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_48_9_generic_type_argument_count_arity_mismatch() {
    // Error: E0216 (GenericArityMismatch)
    let code = r#"
        public class Map<K, V> {
            public K key;
            public V val;
        }

        public class Main {
            public static void test() {
                Map<int32> m = null;
            }
        }
    "#;
    assert_compile_error(code, "E0216");
}

#[test]
fn test_case_48_10_incompatible_type_assignment_between_specialized_generics() {
    // Error: E0203 (TypeMismatch)
    let code = r#"
        public class Item<T> {
            public T value;
            public Item(T v) { this.value = v; }
        }

        public class Main {
            public static void test() {
                Item<int32> a = new Item<bool>(true);
            }
        }
    "#;
    assert_compile_error(code, "E0203");
}

#[test]
fn test_case_48_4_generic_method_with_explicit_type_arguments() {
    let code = r#"
        public class Utils {
            public static T convert<T>(T val) {
                return val;
            }
        }

        public class Main {
            public static int32 main() {
                int32 converted_value = Utils.convert<int32>(55);
                return converted_value == 55 ? 0 : 1;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_48_5_generic_method_with_implicit_template_argument_deduction() {
    let code = r#"
        public class Deduce {
            public static T identity<T>(T item) {
                return item;
            }
        }

        public class Main {
            public static int32 main() {
                int32 deduced_result = Deduce.identity(123);
                return deduced_result == 123 ? 0 : 1;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_48_6_generic_method_with_deductions_from_multiple_arguments() {
    let code = r#"
        public class DeduceMulti {
            public static T selectFirst<T>(T first, T second) {
                return first;
            }
        }

        public class Main {
            public static int32 main() {
                int32 chosen_val = DeduceMulti.selectFirst(77, 88);
                return chosen_val == 77 ? 0 : 1;
            }
        }
    "#;
    let (execution_result, _) = compile_and_run(code);
    assert_eq!(execution_result, Value::I32(0));
}

#[test]
fn test_case_48_11_conflicting_template_deduction_at_call_site() {
    // Diagnostic: E0215 (NoMatchingOverload / Template argument deduction failed)
    let code = r#"
        public class Matcher {
            public static T choose<T>(T first, T second) { return first; }
        }

        public class Main {
            public static void test() {
                Matcher.choose(10, true);
            }
        }
    "#;
    assert_compile_error(code, "E0215");
}

#[test]
fn test_case_48_12_unbound_type_parameter_identifier() {
    // Diagnostic: E0201 (UndefinedSymbol: Unknown type: T)
    let code = r#"
        public class BadGeneric {
            public static void doSomething() {
                T invalidVar = null;
            }
        }
    "#;
    assert_compile_error(code, "E0201");
}
