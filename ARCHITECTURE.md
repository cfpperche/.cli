# Architecture

Status: **M1 shipped** (parser + `cli check`). Everything past M1 is design,
subject to the same "open for debate" rule as [SPEC.md](SPEC.md).

## What gets built here

The repository hosts the **spec and its reference implementation**: a single
binary, `cli`, that runs `.cli` scripts. Components, in dependency order:

### 1. Parser — `crates/parser`

Lexer + recursive-descent parser producing the AST for the grammar in
SPEC.md §3. Stateless; no execution. Errors carry `line:col` and are written
to be *actionable* — agents will generate malformed scripts and must be able
to self-correct from the message alone (e.g. barewords get "strings must be
quoted", not "unexpected token").

Deliberately **zero dependencies**: the grammar is small enough that a
hand-rolled parser gives full control over diagnostics at no cost. If the
grammar outgrows this (string templates, streaming), migrating to `chumsky`
is the designated exit.

### 2. Runtime (M2+)

Tree-walking interpreter over the AST: resolves bindings, moves typed records
through pipes, wraps every command invocation in a result envelope (SPEC §2),
stops at the first error unless `try`. **Effect enforcement lives here**:
before a command runs, its declared effects are checked against the session
policy.

### 3. Stdlib (M2+)

Built-in commands, each born with a manifest (typed params, effects,
idempotency, dry-run support): `log.*`, `glob`, `fs.*`, and the `exec`
bridge. Kept deliberately small — it exists to exercise the model, not to
compete with coreutils.

### 4. Manifest system (M3+)

The data model and loader for command declarations (SPEC §4). Powers
`cli commands`: list everything callable, with types and effects, without
executing anything. External manifests (installed command packs) come later
and carry the supply-chain questions in SPEC §9.

### 5. The `cli` binary — `crates/cli`

The shell around everything:

| Subcommand | Milestone | Does |
| --- | --- | --- |
| `cli check [--ast] <file>…` | M1 ✅ | parse + validate, print diagnostics |
| `cli run <file>` | M2 | execute |
| `cli run --dry-run` | M4 | simulate (SPEC §6) |
| `cli run --deny/--confirm <effect>` | M3 | policy enforcement |
| `cli commands` | M3 | list manifests |

### 6. Agent bridge (M6)

Expose the runtime as an MCP server so any agent can run scripts under a
policy without custom integration. This is where "agent-first" becomes a
product rather than a design stance.

## Stack

Rust, edition 2021, workspace of small crates.

**Dependency policy: start at zero, add on demonstrated need.**

| Need | Crate | When |
| --- | --- | --- |
| Parsing | hand-rolled | now — see rationale above; `chumsky` is the exit |
| Envelopes / JSON | `serde` + `serde_json` | M2, with the runtime |
| CLI flags | hand-rolled | `clap` when the flag surface grows (M3) |
| Tests | `#[test]` + fixtures in `examples/` | `insta` snapshots once the AST stabilizes |
| Sandboxing | in-process checks | OS-level (Landlock/seccomp) is post-M3 research |

Why Rust at all: direct precedent (Nushell, Oils), single static binary, and
a path to real OS-level effect sandboxing without FFI.

## Roadmap

- **M1 ✅ — `cli check`**: lexer, AST, parser, diagnostics; examples parse.
- **M2 — runtime core**: envelopes, `let`/pipes/`try`/`if`, `log.*` + `glob`.
- **M3 — effects**: manifests, effect checks, `--deny`/`--confirm`, `cli commands`.
- **M4 — dry-run**: runtime-wide simulation semantics.
- **M5 — `exec` bridge** + external manifests.
- **M6 — MCP server**.

Each milestone is independently usable — `cli check` already validates
agent-generated scripts with no runtime in existence.

## Repository layout

```
SPEC.md            the minimal language spec (source of truth)
ARCHITECTURE.md    this file
GLOSSARY.md        canonical vocabulary
examples/          scripts in the proposed syntax (also parser fixtures)
crates/
  parser/          lexer + AST + parser (zero deps)
  cli/             the `cli` binary
```
