# AGENTS.md

Instructions for AI agents (Claude Code, Codex, and others) working in this
repository. Humans: start at README.md — this file is optimized for agents,
and practices what SPEC.md §1 goal 5 preaches: only what changes your
behavior, nothing else.

## What this is

A terminal language designed for AI agents and humans as co-equal users
(working title `.cli` — final name undecided, don't bake it into
identifiers). Rust workspace: `crates/parser` (zero-dep lexer + AST +
recursive-descent parser), `crates/cli` (the `cli` binary). Milestone M1
(parser + `cli check`) is shipped; M2 (runtime core) is next. Roadmap:
ARCHITECTURE.md. Language rules: SPEC.md. Vocabulary: GLOSSARY.md.

## Invariants (breaking one of these is the bug)

1. **SPEC.md is the source of truth.** Code implements the spec, never the
   reverse. A syntax or semantics change touches, in the same change:
   SPEC.md §3 grammar, `crates/parser`, `examples/`, and tests.
2. **`examples/*.cli` are parser fixtures.** Tests `include_str!` them; if
   an example stops parsing, the build breaks by design. Adding syntax to an
   example without teaching the parser first will fail CI-less `cargo test`.
3. **Diagnostics are API.** Every parser error must let an agent fix the
   script from the message alone: `line:col`, what is wrong, and a concrete
   fix hint (see the bareword/path/glob errors in `lexer.rs`/`parser.rs` for
   the house style). Tests assert on message content — keep doing that for
   new errors.
4. **GLOSSARY.md is canonical vocabulary.** If code and glossary disagree,
   fix whichever is wrong in the same change.
5. **Dependency policy: zero until demonstrated need.** Planned exits are
   tabled in ARCHITECTURE.md (serde at M2, clap at M3, chumsky only if the
   grammar outgrows the hand-rolled parser). Adding a crate requires
   updating that table with the justification.

## Commands

```console
cargo test                      # build + all tests (parser fixtures included)
cargo fmt --check               # formatting gate (rustfmt defaults, no config)
cargo run -q -p dotcli -- check [--ast] <file>.cli   # validate a script
cargo bench -p dotcli-parser    # parse throughput (see BENCHMARKS.md)
cargo build --release           # ./target/release/cli
```

Definition of done for any change: `cargo test` green, `cargo fmt --check`
clean, `cargo clippy --all-targets -- -D warnings` clean. CI runs exactly
these plus smoke steps (.github/workflows/ci.yml); the regression rules
(bug fix ⇒ failing-first test, new construct ⇒ tests + fixture, new error
⇒ content assertion) live in TESTING.md and are not optional.

## Conventions

- Rust 2021, rustfmt defaults, no rustfmt.toml.
- Repo artifacts (docs, code, commits) in English; project discussion often
  happens in pt-BR, don't be surprised.
- Commit messages: imperative summary, body says *why*; reference milestones
  (M1–M6) when relevant.
- Feature triage uses SPEC.md §1 as a decision rule: safety > readability >
  structured data > boring grammar > context economy — and if an idea serves
  a listed non-goal, the answer is no.

## Current gotchas

- The binary is named `cli` but the package is `dotcli`; the parser package
  is `dotcli-parser` (lib `dotcli_parser`).
- `Statement::If` chains `else if` as a nested `If` inside `else_block`.
- The lexer allows `-` and `.` inside identifiers (`fs.remove`, `dry-run`);
  numbers starting with `-` are disambiguated by lookahead, not by a minus
  token — there is no arithmetic in v0.1.
- No license file yet — the name and the license are both open decisions;
  don't add either unprompted.
