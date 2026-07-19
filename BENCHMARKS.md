# Benchmarks

Two families. The classic one keeps us honest; the second one is where the
thesis is actually tested. Principle: we benchmark against the comparison set
in the README — bash, structured shells, raw MCP tool-calling — never against
strawmen.

## 1. Machine performance (classic)

What a shell must not be slow at. Agents spawn the binary once per script, so
**startup dominates**; scripts are small, so parse must be effectively free.

| Metric | How | Target | Current (M1)¹ |
| --- | --- | --- | --- |
| Cold spawn + check | 200 spawns, wall-clock/​spawn (`hyperfine` when available) | ≤ bash startup | **0.7 ms** (bash `-c true`: 2.9 ms) |
| Parse throughput | `cargo bench -p dotcli-parser` | irrelevant at script sizes | **142 MB/s**, 734 ns/statement |
| Binary size | `ls -la target/release/cli` | single-digit MB | **0.6 MB** |
| Runtime overhead | per-command envelope+pipe cost vs equivalent Nushell pipeline | M2 | — |
| Peak memory | max RSS over the corpus | M2 | — |

¹ WSL2, one machine, no isolation — treat as order-of-magnitude, not truth.
Regression policy once CI exists: parse or startup regressing >10% must be
justified in the PR that causes it.

What we deliberately do **not** chase: data-plane throughput (that is the
commands' job, not the language's) and interactive-prompt latency games.

## 2. Agent performance (the thesis metrics)

A language "for agents" must prove it makes agents *better*, not just feel
safer. These need a model in the loop, so they live in an opt-in eval harness
(`evals/`, lands with M2–M3) — never in CI.

| Metric | Definition | Compared against |
| --- | --- | --- |
| First-try parse rate | % of model-generated scripts that parse on attempt 1 | bash syntax error rate on same tasks |
| **Diagnostic repair rate** | % of parse failures the model fixes in **one** retry given only the `line:col` message | bash's stderr |
| Round-trips per task | model turns to complete a task | bash tool loop; raw MCP calls |
| Tokens per task | prompt+completion tokens, script + envelopes included | same |
| Footgun containment | % of a corpus of classic shell disasters (ported `rm -rf $VAR/`, word-splitting bugs, …) that are **inexpressible or policy-blocked** | bash: 0% by definition |
| Policy false positives | % of safe scripts wrongly blocked under a sane default policy | — (must stay ≈ 0 or nobody uses policies) |
| End-to-end task success | Terminal-Bench-style task suite, same model, `bash` tool vs `cli` tool: success rate, tokens, wall-clock, dangerous-action count | the headline number, eventually |

The diagnostic repair rate is the one the parser is already engineered for —
every error message is written so the fix is derivable from the message alone.
When the eval harness lands, that claim gets a number.

## 3. Reproducing

```console
$ cargo bench -p dotcli-parser        # parse throughput
$ cargo build --release
$ time (for i in $(seq 1 200); do ./target/release/cli check examples/publish.cli >/dev/null; done)
```
