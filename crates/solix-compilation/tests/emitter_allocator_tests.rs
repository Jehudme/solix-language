//! Unit and integration tests for Phase 2: Register Allocator & Constant Pool.

use solix_common::diagnostics::EmissionError;
use solix_compilation::phases::emitter::{
    ConstantPoolBuilder, MAX_ARGUMENTS_PER_CALL, RegisterAllocator,
};

#[test]
fn test_parameter_register_assignment() {
    // Instance method with `this` at r0 and two parameters (r1, r2)
    let mut allocator = RegisterAllocator::new(3);
    assert_eq!(allocator.param_count(), 3);
    assert_eq!(allocator.peak_registers(), 3);

    let this_reg = allocator.alloc_param("this").expect("alloc this");
    let a_reg = allocator.alloc_param("param_a").expect("alloc param_a");
    let b_reg = allocator.alloc_param("param_b").expect("alloc param_b");

    assert_eq!(this_reg, 0);
    assert_eq!(a_reg, 1);
    assert_eq!(b_reg, 2);

    assert_eq!(allocator.lookup_local("this"), Some(0));
    assert_eq!(allocator.lookup_local("param_a"), Some(1));
    assert_eq!(allocator.lookup_local("param_b"), Some(2));
    assert_eq!(allocator.lookup_local("undefined"), None);
}

#[test]
fn test_disjoint_sibling_scope_register_reuse() {
    // Function with 0 parameters
    let mut allocator = RegisterAllocator::new(0);

    // Scope 1: { int a; int b; }
    allocator.enter_scope();
    let reg_a = allocator.alloc_local("a").expect("alloc a");
    let reg_b = allocator.alloc_local("b").expect("alloc b");
    assert_eq!(reg_a, 0);
    assert_eq!(reg_b, 1);
    assert_eq!(allocator.peak_registers(), 2);
    allocator.exit_scope();

    // After exiting Scope 1, variables 'a' and 'b' are no longer in scope
    assert_eq!(allocator.lookup_local("a"), None);
    assert_eq!(allocator.lookup_local("b"), None);

    // Scope 2: { int c; int d; } - MUST reuse registers 0 and 1!
    allocator.enter_scope();
    let reg_c = allocator.alloc_local("c").expect("alloc c");
    let reg_d = allocator.alloc_local("d").expect("alloc d");
    assert_eq!(reg_c, 0);
    assert_eq!(reg_d, 1);
    // Peak register count should remain 2, not 4!
    assert_eq!(allocator.peak_registers(), 2);
    allocator.exit_scope();
}

#[test]
fn test_nested_lexical_scopes_and_shadowing() {
    let mut allocator = RegisterAllocator::new(1); // r0 is param 'x'
    allocator.alloc_param("x").unwrap();

    allocator.enter_scope();
    let outer_y = allocator.alloc_local("y").unwrap();
    assert_eq!(outer_y, 1);

    // Inner scope
    allocator.enter_scope();
    let inner_z = allocator.alloc_local("z").unwrap();
    assert_eq!(inner_z, 2);

    // Inner scope can see outer variables
    assert_eq!(allocator.lookup_local("x"), Some(0));
    assert_eq!(allocator.lookup_local("y"), Some(1));
    assert_eq!(allocator.lookup_local("z"), Some(2));

    // Shadow 'y' in innermost scope
    let inner_y = allocator.alloc_local("y").unwrap();
    assert_eq!(inner_y, 3);
    assert_eq!(allocator.lookup_local("y"), Some(3));

    // Exit inner scope: shadowed 'y' and 'z' are popped
    allocator.exit_scope();
    assert_eq!(allocator.lookup_local("z"), None);
    assert_eq!(allocator.lookup_local("y"), Some(1)); // Restored to outer_y!

    allocator.exit_scope();
    assert_eq!(allocator.lookup_local("y"), None);
}

#[test]
fn test_temporary_scratch_register_recycling() {
    let mut allocator = RegisterAllocator::new(0);

    // Simulating expression evaluation: (a + b) * (c + d)
    // 1. eval (a + b) -> temp1 (r0)
    let temp1 = allocator.alloc_temp().expect("alloc temp1");
    assert_eq!(temp1, 0);

    // 2. eval (c + d) -> temp2 (r1)
    let temp2 = allocator.alloc_temp().expect("alloc temp2");
    assert_eq!(temp2, 1);

    // 3. Multiply temp1 and temp2 into temp3 (recycle temp1 as dst)
    allocator.free_temp(temp2);
    // Next temp allocation should reuse temp2 (r1)
    let temp3 = allocator.alloc_temp().expect("alloc temp3");
    assert_eq!(temp3, 1);

    allocator.free_temp(temp1);
    allocator.free_temp(temp3);

    // Next temps should reuse r0 and r1
    let reused_0 = allocator.alloc_temp().expect("reuse r0");
    let reused_1 = allocator.alloc_temp().expect("reuse r1");
    assert_eq!(reused_0, 0);
    assert_eq!(reused_1, 1);
    assert_eq!(allocator.peak_registers(), 2);
}

#[test]
fn test_contiguous_argument_window_allocation() {
    let mut allocator = RegisterAllocator::new(2); // r0, r1 are params
    let _ = allocator.alloc_local("loc").unwrap(); // r2 is local

    // Window of 4 arguments for call foo(a, b, c, d)
    let window_base = allocator.alloc_window(4).expect("alloc window 4");
    assert_eq!(window_base, 3); // Must start immediately after r2

    // Peak registers should now be at least 7 (r0..r6)
    assert!(allocator.peak_registers() >= 7);

    // Free the window
    allocator.free_window(window_base, 4);

    // Next window of 3 arguments
    let window_base_2 = allocator.alloc_window(3).expect("alloc window 3");
    assert_eq!(window_base_2, 7); // Fresh contiguous range
    allocator.free_window(window_base_2, 3);
}

#[test]
fn test_window_argument_limit_overflow() {
    let mut allocator = RegisterAllocator::new(0);
    let overflow_count = MAX_ARGUMENTS_PER_CALL + 1; // 256 arguments

    let result = allocator.alloc_window(overflow_count);
    assert!(result.is_err());
    match result {
        Err(EmissionError::ArgumentCountOverflow {
            argument_count,
            max,
        }) => {
            assert_eq!(argument_count, 256);
            assert_eq!(max, 255);
        }
        other => panic!("Expected ArgumentCountOverflow, got {other:?}"),
    }
}

#[test]
fn test_constant_pool_builder_lifecycle() {
    let mut builder = ConstantPoolBuilder::new();
    assert!(builder.is_empty());

    let i1 = builder.add_i64(999).expect("add i64");
    let i2 = builder.add_i64(999).expect("dedup i64");
    assert_eq!(i1, i2);

    let f1 = builder.add_f64(123.456).expect("add f64");
    let f2 = builder.add_f64(123.456).expect("dedup f64");
    assert_eq!(f1, f2);

    let s1 = builder.add_string("compiler").expect("add string");
    let s2 = builder.add_string("compiler").expect("dedup string");
    assert_eq!(s1, s2);

    assert_eq!(builder.len(), 3);
    assert!(!builder.is_empty());

    let pool = builder.finish();
    assert_eq!(pool.len(), 3);
}

#[test]
fn test_parameter_bounds_and_no_overlap_under_heavy_register_pressure() {
    let parameter_count: u16 = 8;
    let mut allocator = RegisterAllocator::new(parameter_count);
    assert_eq!(allocator.param_count(), parameter_count);

    let mut parameter_registers = Vec::new();
    for parameter_index in 0..parameter_count {
        let parameter_name = format!("param_{parameter_index}");
        let allocated_param = allocator.alloc_param(&parameter_name).expect("alloc param");
        assert_eq!(allocated_param, parameter_index);
        parameter_registers.push(allocated_param);
    }

    // Enter multiple nested scopes and allocate local variables
    let mut local_registers = Vec::new();
    for scope_depth in 0..5 {
        allocator.enter_scope();
        for local_index in 0..12 {
            let local_name = format!("local_s{scope_depth}_{local_index}");
            let allocated_local = allocator.alloc_local(&local_name).expect("alloc local");
            assert!(
                allocated_local >= parameter_count,
                "Local register {allocated_local} must not overlap parameter space"
            );
            local_registers.push(allocated_local);
        }
    }

    // Allocate heavy scratch temporaries simulating nested expression trees
    let mut temporary_registers = Vec::new();
    for _ in 0..64 {
        let temp_reg = allocator.alloc_temp().expect("alloc temp");
        assert!(
            temp_reg >= parameter_count,
            "Temp register {temp_reg} must not overlap parameter space"
        );
        temporary_registers.push(temp_reg);
    }

    // Scramble deallocations and window reservations
    for temp_reg in temporary_registers.drain(..32) {
        allocator.free_temp(temp_reg);
    }

    let window_base = allocator.alloc_window(16).expect("alloc window 16");
    assert!(
        window_base >= parameter_count,
        "Call window base {window_base} must not overlap parameter space"
    );
    allocator.free_window(window_base, 16);

    // Pop nested scopes
    for _ in 0..5 {
        allocator.exit_scope();
    }

    // Re-allocate temporaries and locals after scope reclamation
    for _ in 0..100 {
        let recycled_temp = allocator.alloc_temp().expect("alloc temp");
        assert!(
            recycled_temp >= parameter_count,
            "Recycled register {recycled_temp} must never overlap parameter range 0..{parameter_count}"
        );
        allocator.free_temp(recycled_temp);
    }

    // Verify all original parameters still resolve correctly
    for parameter_index in 0..parameter_count {
        let parameter_name = format!("param_{parameter_index}");
        assert_eq!(
            allocator.lookup_local(&parameter_name),
            Some(parameter_index)
        );
    }
}
