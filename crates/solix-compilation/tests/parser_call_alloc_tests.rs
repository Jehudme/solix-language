//! Allocation regression tests verifying valid call statements incur zero extra heap allocations
//! during parser statement discrimination (`#306`).

use solix_compilation::phases::lexing::lex_source;
use solix_compilation::phases::parser::parse_source;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct CountingAllocator;

static TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);

#[allow(unsafe_code)]
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

fn measure_parse_allocations(call_identifier: &str, repetitions: usize) -> usize {
    let mut source = String::from("public class BenchmarkApp {\n    public static void run() {\n");
    for _ in 0..repetitions {
        source.push_str("        ");
        source.push_str(call_identifier);
        source.push_str("(1);\n");
    }
    source.push_str("    }\n}\n");

    let tokens = lex_source(0, &source).expect("Lexing test source must succeed");

    // Warm up allocator / lazily initialized globals if any on a dummy parse first
    let warmup_tokens = lex_source(0, "public class Warmup { public static void run() {} }")
        .expect("Warmup lex succeeds");
    let _ = parse_source(
        0,
        &warmup_tokens,
        "public class Warmup { public static void run() {} }",
    );

    ALLOCATION_COUNT.store(0, Ordering::SeqCst);
    TRACKING_ENABLED.store(true, Ordering::SeqCst);
    let parsed = parse_source(0, &tokens, &source);
    TRACKING_ENABLED.store(false, Ordering::SeqCst);

    assert!(parsed.is_ok(), "Expected valid calls to parse cleanly");
    ALLOCATION_COUNT.load(Ordering::SeqCst)
}

#[test]
fn test_valid_call_statements_incur_zero_extra_levenshtein_allocations() {
    let baseline_long_lowercase = measure_parse_allocations("long_lowercase_call", 200);
    let short_lowercase = measure_parse_allocations("work", 200);
    let uppercase_method = measure_parse_allocations("RenderFrame", 200);

    assert_eq!(
        short_lowercase, baseline_long_lowercase,
        "Short lowercase valid calls ('work(1);') must incur zero extra allocations vs long names ({short_lowercase} vs {baseline_long_lowercase})"
    );
    assert_eq!(
        uppercase_method, baseline_long_lowercase,
        "PascalCase valid calls ('RenderFrame(1);') must incur zero extra allocations ({uppercase_method} vs {baseline_long_lowercase})"
    );
}
