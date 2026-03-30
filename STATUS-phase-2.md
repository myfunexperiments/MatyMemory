# Phase 2: CLI Commands + Shared Executor Layer

## Task Progress

### Serde Dependencies + Serialize Derives
- Status: complete

### New Store Methods (delete, change_type, get_provenance)
- Status: complete

### Config Module (DB path resolution)
- Status: complete

### Executor Layer
- Status: complete

### CLI Parsing + Output
- Status: complete

### Integration Tests
- Status: complete

### Review
- Status: complete (issues found, fixes required)

---

## Review Findings

### Tests
- All 58 tests pass (38 unit + 20 integration).

### Clippy (`cargo clippy -- -D warnings`)
- **FAILS** with 17 errors. Phase 2 issues that must be fixed:
  - `src/config.rs:8` -- collapsible if (nested `if let` + `if`)
  - `src/executor/remember.rs:7` -- too many arguments (10/7)
  - `src/store/memory.rs:208` -- redundant closure (`s.map(|s| parse_ts(s))` should be `s.map(&parse_ts)`)
  - `src/store/models.rs:98` -- derivable Default impl for `Update<T>`
- Pre-existing issues (not blockers for Phase 2 but should be cleaned up):
  - Dead code warnings for `open_in_memory`, `Update::Null`, `RetrievalLogEntry`, `Scope`, `SELECT_MEMORY_STATUS`, `DELETE_RELATION`, `INSERT_RETRIEVAL_LOG`, `remove_relation`, `log_retrieval`, `batch_get_tags`
  - `src/repl.rs` -- while_let_loop, collapsible_if, unnecessary_map_or

### File Sizes (200-line limit)
- `tests/cli_integration.rs` is **472 lines** -- exceeds limit. Should be split into multiple test files.
- All other source files are within limits.

### Bugs Found (confirmed by Codex)

#### BUG-1 (HIGH): UTF-8 panic in truncate -- `src/cli/output.rs:130`
- `&s[..max_len]` slices by byte offset, which panics if the cut lands inside a multi-byte UTF-8 character.
- Repro: store 59 ASCII chars + an emoji, then run `list`.
- Fix: use `s.char_indices()` to find the correct boundary, or use `s.chars().take(max_len)`.

#### BUG-2 (HIGH): `forget` fails on superseded memories -- `src/store/memory.rs:134`
- `DELETE FROM memories WHERE id = ?1` fails with a FOREIGN KEY error when another memory's `supersedes_id` references the deleted memory.
- Repro: `remember old`, `remember new`, `supersede old new`, `forget old` -> FK constraint failure.
- Fix: either NULL out `supersedes_id` references before deleting, or add `ON DELETE SET NULL` to the schema for `supersedes_id`.

#### BUG-3 (MEDIUM): DB open error bypasses JSON output -- `src/main.rs:35-38`
- When `--json` is set but `Store::open` fails, the error is printed as plain text, not JSON. Machine consumers get non-JSON on startup errors.
- Fix: check `cli.json` in the `Store::open` error branch and format accordingly.

### Architecture and Design (PASS)
- Executor layer is clean -- no CLI/formatting logic leaked in.
- All Store errors are properly propagated via `?`.
- JSON output is valid and parseable (verified by integration tests).
- Error messages are clear ("Memory not found: <id>", "Unknown memory type: <x>").
- Config resolution order (--db > MATY_DB_PATH > ~/.matymemory/memory.db) is correct and tested.

---

## Verdict

**Phase 2 does NOT pass review.** Three fixes are required before proceeding:

1. Fix the UTF-8 truncation panic (BUG-1)
2. Fix the FK constraint error on `forget` for superseded memories (BUG-2)
3. Fix the JSON error formatting for DB open failures (BUG-3)
4. Fix clippy errors in Phase 2 files (config.rs, executor/remember.rs, store/memory.rs, store/models.rs)
5. Split `tests/cli_integration.rs` (472 lines) to meet the 200-line limit

After these fixes, the phase can be re-reviewed and approved.
