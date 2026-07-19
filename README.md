# .cli

> **Working title.** The final name is still under discussion — `.cli` is the file
> extension and the placeholder project name for now.

A terminal language designed for **AI agents and humans as co-equal users**.

## Why

Existing shells were designed for humans typing as few characters as possible.
That design is hostile to AI agents (and, honestly, to humans too):

- Fragile quoting and implicit word splitting (`rm -rf $DIR/` disasters)
- Unstructured text output that must be scraped with regex
- Errors as free-form text on stderr, with no semantic code
- No way to know, before running a command, whether it reads, writes,
  touches the network, or destroys data
- No first-class dry-run, no declared idempotency — an agent can never know
  what is safe to retry

Structured-data shells (Nushell, PowerShell, Oils/YSH) fix the *output* problem.
None of them are **agent-first**: none let a command declare its effects, its
idempotency, or its dry-run behavior in a way a runtime can enforce and an
agent can reason about.

That is the gap this project targets.

## Core ideas

1. **Declared effects** — every command states what it can do (`read`, `write`,
   `net`, `destructive`). The runtime enforces it; agents and permission
   systems can reason about it *before* execution.
2. **First-class dry-run** — `--dry-run` is runtime semantics, not a convention
   each tool may or may not honor.
3. **Structured everything** — output, errors, and progress are typed records,
   not text. Human rendering is a presentation layer on top.
4. **Declared idempotency** — a command says whether re-running it is safe, so
   retry logic stops being guesswork.
5. **Unambiguous grammar** — no aliases, no implicit expansion, no
   context-dependent parsing. One way to read every line.

## Status

🚧 **Draft spec, early implementation.** The spec ([SPEC.md](SPEC.md)) is the
source of truth and everything in it is open for debate. Milestone M1 of the
reference implementation is done: a zero-dependency parser and `cli check`.

```console
$ cargo build
$ ./target/debug/cli check examples/publish.cli
examples/publish.cli: OK (6 statements, 7 commands)

$ echo 'fs.remove /tmp/foo' | tee /tmp/bad.cli >/dev/null; ./target/debug/cli check /tmp/bad.cli
/tmp/bad.cli:1:11: unexpected `/` — paths are strings and must be quoted (e.g. "/tmp/build")
```

Diagnostics are written so an agent can self-correct from the message alone —
that is the project thesis applied to its own tooling.

## Repository layout

- [`SPEC.md`](SPEC.md) — the minimal language specification (v0.1 draft)
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — components, stack, roadmap
- [`GLOSSARY.md`](GLOSSARY.md) — canonical vocabulary
- [`examples/`](examples/) — scripts in the proposed syntax (also parser fixtures)
- [`crates/parser`](crates/parser/) — lexer + AST + parser (zero deps)
- [`crates/cli`](crates/cli/) — the `cli` binary (`check` today; `run` next)
