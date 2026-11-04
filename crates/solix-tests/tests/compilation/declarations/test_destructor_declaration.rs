//! Suite: DestructorDeclaration - Declarations (TESTS.md)

use solix_runtime::Value;
use solix_tests::common::{assert_compile_error, compile_and_run};

#[test]
fn test_case_3_1_explicit_dereference_raii_cleanup() {
    let code = r#"
        class Tracker {
            public static int32 destroyed_count = 0;
            ~Tracker() {
                Tracker::destroyed_count = Tracker::destroyed_count + 1;
            }
        }

        class Main {
            public static int32 main() {
                Tracker::destroyed_count = 0;
                Tracker t = new Tracker();
                if (Tracker::destroyed_count != 0) return 1;
                t = null;
                if (Tracker::destroyed_count != 1) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_2_block_scope_exit_raii_cleanup() {
    let code = r#"
        class Resource {
            public static int32 counter = 0;
            ~Resource() {
                Resource::counter = Resource::counter + 1;
            }
        }

        class Main {
            public static int32 main() {
                Resource::counter = 0;
                {
                    Resource r1 = new Resource();
                    Resource r2 = new Resource();
                    if (Resource::counter != 0) return 1;
                }
                if (Resource::counter != 2) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_3_function_return_raii_cleanup() {
    let code = r#"
        class Guard {
            public static int32 count = 0;
            ~Guard() {
                Guard::count = Guard::count + 1;
            }
        }

        class Main {
            public static int32 doWork() {
                Guard g = new Guard();
                return 42;
            }

            public static int32 main() {
                Guard::count = 0;
                int32 val = Main::doWork();
                if (val != 42) return 1;
                if (Guard::count != 1) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_4_subclass_to_base_inheritance_chaining() {
    let code = r#"
        class BaseResource {
            public static int32 sequence = 0;
            ~BaseResource() {
                BaseResource::sequence = BaseResource::sequence * 10 + 1;
            }
        }

        class DerivedResource extends BaseResource {
            ~DerivedResource() {
                BaseResource::sequence = BaseResource::sequence * 10 + 2;
            }
        }

        class Main {
            public static int32 main() {
                BaseResource::sequence = 0;
                {
                    DerivedResource d = new DerivedResource();
                }
                if (BaseResource::sequence != 21) return 1;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_5_multi_level_inheritance_chaining() {
    let code = r#"
        class Grandparent {
            public static int32 order = 0;
            ~Grandparent() {
                Grandparent::order = Grandparent::order * 10 + 1;
            }
        }

        class Parent extends Grandparent {
            ~Parent() {
                Grandparent::order = Grandparent::order * 10 + 2;
            }
        }

        class Child extends Parent {
            ~Child() {
                Grandparent::order = Grandparent::order * 10 + 3;
            }
        }

        class Main {
            public static int32 main() {
                Grandparent::order = 0;
                {
                    Child c = new Child();
                }
                if (Grandparent::order != 321) return 1;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_6_field_and_state_mutation_within_destructor() {
    let code = r#"
        class Logger {
            public static int32 last_logged = 0;
            public int32 id;
            public Logger(int32 id) { this.id = id; }
            ~Logger() {
                Logger::last_logged = this.id;
            }
        }

        class Main {
            public static int32 main() {
                {
                    Logger l = new Logger(777);
                }
                if (Logger::last_logged != 777) return 1;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_7_cascading_destruction_of_nested_resources() {
    let code = r#"
        class InnerNode {
            public static int32 count = 0;
            ~InnerNode() {
                InnerNode::count = InnerNode::count + 1;
            }
        }

        class OuterContainer {
            public InnerNode node;
            public OuterContainer() {
                this.node = new InnerNode();
            }
            ~OuterContainer() {
                this.node = null;
            }
        }

        class Main {
            public static int32 main() {
                InnerNode::count = 0;
                {
                    OuterContainer c = new OuterContainer();
                }
                if (InnerNode::count != 1) return 1;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_8_exception_rewinding_raii_across_multiple_call_frames() {
    let code = r#"
        class Exception {}

        class AutoCleanup {
            public static int32 cleaned_frames = 0;
            ~AutoCleanup() {
                AutoCleanup::cleaned_frames = AutoCleanup::cleaned_frames + 1;
            }
        }

        class Runner {
            public static void levelThree() {
                AutoCleanup c3 = new AutoCleanup();
                throw new Exception();
            }

            public static void levelTwo() {
                AutoCleanup c2 = new AutoCleanup();
                Runner::levelThree();
            }

            public static void levelOne() {
                AutoCleanup c1 = new AutoCleanup();
                Runner::levelTwo();
            }
        }

        class Main {
            public static int32 main() {
                AutoCleanup::cleaned_frames = 0;
                try {
                    Runner::levelOne();
                } catch (Exception exc) {
                    if (AutoCleanup::cleaned_frames != 3) return 1;
                    return 0;
                }
                return 2;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_9_exception_in_try_block_raii() {
    let code = r#"
        class Exception {}

        class ScopedLock {
            public static bool unlocked = false;
            ~ScopedLock() {
                ScopedLock::unlocked = true;
            }
        }

        class Main {
            public static int32 main() {
                ScopedLock::unlocked = false;
                try {
                    ScopedLock lock_guard = new ScopedLock();
                    throw new Exception();
                } catch (Exception exc) {
                    if (!ScopedLock::unlocked) return 1;
                    return 0;
                }
                return 2;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_10_variable_and_field_reassignment_cleanup() {
    let code = r#"
        class Cell {
            public static int32 counter = 0;
            ~Cell() {
                Cell::counter = Cell::counter + 1;
            }
        }

        class Main {
            public static int32 main() {
                Cell::counter = 0;
                Cell c = new Cell();
                if (Cell::counter != 0) return 1;
                c = new Cell();
                if (Cell::counter != 1) return 2;
                c = null;
                if (Cell::counter != 2) return 3;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_3_11_array_of_destructible_objects_cleanup() {
    let code = r#"
        class Item {
            public static int32 count = 0;
            ~Item() {
                Item::count = Item::count + 1;
            }
        }

        class Main {
            public static int32 main() {
                Item::count = 0;
                {
                    Item[] arr = new Item[3];
                    arr[0] = new Item();
                    arr[1] = new Item();
                    arr[2] = new Item();
                    if (Item::count != 0) return 1;
                }
                if (Item::count != 3) return 2;
                return 0;
            }
        }
    "#;
    let (res, _) = compile_and_run(code);
    assert_eq!(res, Value::I32(0));
}

#[test]
fn test_case_4_1_destructor_declared_with_parameters() {
    // Error: E0100 (Syntax / parameter error)
    let code = r#"
        class Widget {
            ~Widget(int32 x) {}
        }
    "#;
    assert_compile_error(code, "parameter");
}

#[test]
fn test_case_4_2_destructor_declared_with_explicit_return_type() {
    // Error: E0211 (ReturnOutsideFunction)
    let code = r#"
        class Widget {
            ~Widget() {
                return 10;
            }
        }
    "#;
    assert_compile_error(code, "E0211");
}

#[test]
fn test_case_4_3_duplicate_destructors_in_same_class() {
    // Error: E0202 (DuplicateSymbol)
    let code = r#"
        class Widget {
            ~Widget() {}
            ~Widget() {}
        }
    "#;
    assert_compile_error(code, "destructor");
}

#[test]
fn test_case_4_4_destructor_name_mismatch() {
    // Error: E0131 (DestructorNameMismatch)
    let code = r#"
        class Widget {
            public ~Other() {}
        }
    "#;
    assert_compile_error(code, "E0131");
}

#[test]
fn test_case_4_5_destructor_in_interface() {
    // Error: E0125 (InterfaceDestructor)
    let code = r#"
        interface IService {
            ~IService();
        }
    "#;
    assert_compile_error(code, "E0125");
}

#[test]
fn test_case_4_6_destructor_declared_with_forbidden_modifiers() {
    // Error: E0130 (InvalidDestructorModifier)
    let code = r#"
        class Widget {
            public static ~Widget() {}
        }
    "#;
    assert_compile_error(code, "E0130");
}
