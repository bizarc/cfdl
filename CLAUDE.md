You are an implementation agent for the CFDL v0.1 compiler workspace.

Read and follow:
- CLAUDE.md (operating rules)
- AGENTS.md (milestones + acceptance criteria)
- @docs/* (authoritative specs)

TASK (Milestone 1 — Lexer scaffolding):
1) Implement the lexer in crates/cfdl-lexer per @docs/compiler_spec_v0_1.md and docs/CFDL_v0_1_Grammar.ebnf:
   - Tokenize identifiers, keywords, punctuation, string literals, numeric literals, date literals.
   - Support line comments // and block comments /* */.
   - Produce spans (start_line/start_col/end_line/end_col) for every token and for lexer diagnostics.
   - Deterministic behavior only.

2) Add TWO invalid fixtures and golden diagnostics:
   - fixtures/invalid/lex_unterminated_string/model.cfdl
   - fixtures/invalid/lex_unterminated_block_comment/model.cfdl
   The expected error codes must match @docs/diagnostics_spec.md (E0002_UNTERMINATED_STRING and E0003_UNTERMINATED_BLOCK_COMMENT). Ensure diagnostics include file and span.

3) Wire the CLI parse command minimally ONLY if required for golden tests. Prefer unit tests in crates/cfdl-lexer. If you do touch the CLI, keep it thin and do not change the existing CLI interface (compile/validate contract) or diagnostic code meanings.

DONE WHEN:
- make fmt && make lint && make test && make gold all pass
- golden outputs are updated ONLY via CFDL_GOLD_UPDATE=1 when generating new gold files
- No changes to existing diagnostic codes; only add if unavoidable.

Deliverables in the PR:
- lexer implementation in crates/cfdl-lexer
- the two fixture directories + their gold/diag/*.diag.json files
- brief notes describing token model + how spans are computed