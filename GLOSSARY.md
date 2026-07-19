# Glossary

Canonical vocabulary for the spec, the code, and issue discussions. When a
term here and the code disagree, one of them is a bug.

| Term | Definition |
| --- | --- |
| **Script** | A `.cli` file; a sequence of statements executed top to bottom. |
| **Pipeline** | One or more commands joined by `\|` (optionally fed by a value, e.g. `$pages \| md.render`). What flows between them is typed records, never bytes. |
| **Command** | A callable unit (`fs.remove`, `glob`). Exists only if declared in a manifest — there is no `$PATH` discovery. |
| **Binding** | `let name = pipeline` — captures a pipeline's value in an immutable variable. |
| **Record** | A structured key→value datum; the data type that crosses pipes. |
| **Envelope** | The result of every invocation: `{status, value \| error, effects}`. There is no "loose" output. |
| **Effect** | A command's declared capability: `read`, `write`, `net`, `destructive`. The runtime enforces it; a violation is an error, not a warning. |
| **Policy** | Session rules over effects: `--deny destructive`, `--confirm net`. Decided before execution, not during. |
| **Manifest** | The machine-readable declaration of a command: typed params, effects, idempotency, dry-run support, output shape. |
| **Stdlib** | Commands built into the binary (`fs.*`, `glob`, `log.*`, `exec`). |
| **`exec` (bridge)** | The only path to external binaries; declares worst-case effects, making escapes from the model visible and auditable. |
| **Dry-run** | A *runtime* mode: each command either simulates (`supported`), runs normally because it only reads (`inherent`), or fails visibly (`unsupported`). |
| **Idempotent** | Manifest flag: re-running is safe. Combined with an error's `retryable`, retry policy becomes decidable: retry iff `retryable && idempotent`. |
| **`try`** | Converts an error envelope into a value — error handling becomes data flow, not control-flow acrobatics. |
| **Error code** | Stable, namespaced identifier (`fs/not-found`); it is API. `message` and `hint` are presentation and may change freely. |
| **Bareword** | An unquoted string where a value is expected. Illegal by design; the parser rejects it with a fix-it hint. |
| **AST** | The tree the parser produces from a script; the runtime's only input — no re-parsing, no string evaluation downstream. |
