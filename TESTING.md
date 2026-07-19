# Testing & Regression Strategy

Principle: a regression must be **structurally hard to land**, not caught by
heroics. Every layer below exists to make a class of drift impossible, and
the layers are cheap enough that skipping them is never justified.

## Layers (in force today)

1. **Examples are fixtures.** Every shipped example in `examples/` is
   `include_str!`-ed into the parser tests. Docs and parser cannot drift
   apart: syntax shown to humans is syntax the parser proves it accepts, on
   every `cargo test`.
2. **Diagnostics are tested by content.** The message contract —
   `line:col`, what is wrong, a concrete fix hint — is asserted with
   substring checks (`must be quoted`, `glob command`, …). Error quality
   cannot silently rot; changing a message intentionally means changing its
   test in the same commit.
3. **Unit tests per grammar construct.** Each construct (bindings, flags,
   lists, records, `try`, `if`/`else`, pipe continuation, negative numbers,
   comments) has at least one positive test; footguns have negative tests.
4. **Static gates.** `cargo fmt --check` and
   `cargo clippy --all-targets -- -D warnings` — zero tolerated warnings, so
   warning count never becomes a creeping baseline.
5. **CI on every push and PR** (`.github/workflows/ci.yml`): the gates
   above, `cargo test`, a release build, and two smoke steps — the release
   binary must `check` all examples, and the parse bench must run. Bench
   *numbers* are informational in CI (shared runners are too noisy to gate
   on); the >10% regression rule in BENCHMARKS.md §1 is applied on a quiet
   machine when a perf-sensitive change lands.

## Rules for contributions (agents included)

- **Every bug fix lands with a test that failed before the fix.** Name it
  after the bug, not after the function.
- **Every new grammar construct lands with**: positive tests, at least one
  diagnostic test for its failure mode, and an example (or test fixture)
  exercising it end to end.
- **Every new error message gets a content assertion** the moment it is
  written — the diagnostic contract (AGENTS.md invariant 3) is enforced by
  test, not by review vigilance.
- Definition of done, always: `cargo test` green, `cargo fmt --check` clean,
  `cargo clippy --all-targets -- -D warnings` clean.

## Planned expansion (tracks the roadmap)

| When | What | Why |
| --- | --- | --- |
| M2 | Golden envelope tests: script in → JSON envelopes out, snapshot-compared (`insta` enters here) | runtime output becomes API the moment it exists |
| M2 | OS matrix in CI (macOS, Windows) once the fs stdlib lands | path semantics are a regression class of their own |
| M3 | **Footgun corpus**: ported bash disasters asserted *inexpressible or policy-blocked* | doubles as the safety benchmark (BENCHMARKS.md §2) |
| M3+ | Opt-in eval harness (`evals/`) for agent metrics — never in CI | model-in-the-loop, expensive, nondeterministic |
| Multi-contributor | Branch protection + required-PR flow with CI as required check | today: solo direct-push with the local DoD; flip when a second contributor appears |
