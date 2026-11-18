# Control Flow Statement Specifications

Control-flow graph structure, ARC scope cleanup, jump emission, and exception table generation.

- [`block_statement.md`](block_statement.md) — Lexical scope boundaries and automatic ARC cleanup
- [`variable_declaration.md`](variable_declaration.md) — Local slot allocation for scalar and reference types
- [`expression_statement.md`](expression_statement.md) — Side-effect evaluation and temporary reference drops
- [`if_statement.md`](if_statement.md) — Predicate branching and short-circuit lowering
- [`while_statement.md`](while_statement.md) — Pre-tested loop emission
- [`do_while_statement.md`](do_while_statement.md) — Post-tested single-execution loop emission
- [`for_statement.md`](for_statement.md) — Counted loops and `ForLoopI32` superinstruction lowering
- [`switch_statement.md`](switch_statement.md) — Multi-way case matching and fall-through rules
- [`break_statement.md`](break_statement.md) — Non-local loop/switch exit and intermediate ARC release
- [`continue_statement.md`](continue_statement.md) — Loop iteration advancement and scope cleanup
- [`return_statement.md`](return_statement.md) — Frame exit, owned return optimization, and parameter release
- [`throw_statement.md`](throw_statement.md) — Exception raising and stack unwinding entry
- [`try_catch_finally_statement.md`](try_catch_finally_statement.md) — Zero-cost exception handler tables and `finally` guarantees
- [`intrinsics.md`](intrinsics.md) — Built-in `assert` and `exit` runtime intrinsics
