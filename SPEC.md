# Minimal Specification — v0.1 (draft)

Status: **draft, everything open for debate.**
Scope: the smallest set of decisions that makes the idea concrete enough to
criticize. Not a full language reference.

---

## 1. Design goals

In priority order:

1. **Safe for agents.** An agent must be able to know, before execution, what a
   command can touch and whether it is safe to retry.
2. **Readable by humans.** A human reviewing an agent's script must understand
   it without a manual. Verbosity is acceptable; ambiguity is not.
3. **Machine-parseable end to end.** Output, errors, and progress are typed
   data. Text rendering is a presentation concern, never the source of truth.
4. **Boring to parse.** The grammar is context-free and small. No mode
   switches, no implicit expansion, no runtime-dependent parsing.
5. **Context-frugal.** The scarce resource is the agent's context window,
   not the token bill — prices fall with technology; attention does not.
   Polluted context degrades an agent's reasoning even when tokens are free.
   Every feature is judged by what it forces into the agent's working
   context: one script instead of N tool-call round-trips; envelopes compact
   by default, verbose on demand; diagnostics good enough that repair takes
   one retry; manifests browsable without dumping every schema. Raw token
   count still matters, but as a secondary concern, never a maximalist one.
   Never syntax golf: when brevity and readability conflict, goal 2 wins.
   The metric remains *tokens per task* (BENCHMARKS.md §2) — the best
   measurable proxy for context load today.

Non-goals (for now): interactive ergonomics (completions, prompts, job
control) — this is a script substrate, not a daily-driver shell; POSIX
compatibility; being a general-purpose programming language — complex logic
belongs inside commands, not in the language; machine-benchmark supremacy —
startup and parse targets are hygiene (BENCHMARKS.md §1), not the product.

## 2. Execution model

A script is a sequence of **pipelines**. A pipeline is a sequence of
**commands** connected by `|`. Values flowing through a pipe are **typed
records**, not byte streams.

Every command invocation produces exactly one **result envelope**:

```
{
  "status": "ok" | "error",
  "value": <any>,          # present when status = ok
  "error": <Error>,        # present when status = error
  "effects": [<Effect>],   # what actually happened (see §5)
}
```

A pipeline stops at the first `error` unless explicitly told otherwise
(`try` — see §7).

## 3. Grammar (minimal)

```ebnf
script      = { statement } ;
statement   = pipeline | binding | conditional | comment ;
binding     = "let" IDENT "=" pipeline ;
conditional = "if" condition block [ "else" ( block | conditional ) ] ;
block       = "{" { statement } "}" ;
condition   = value [ ( "==" | "!=" ) value ] ;
pipeline    = [ "try" ] ( command | value ) { "|" command } ;
command     = IDENT { argument } ;
argument    = named_arg | value ;
named_arg   = "--" IDENT [ "=" value ] ;
value       = STRING | NUMBER | BOOL | list | record | var_ref ;
list        = "[" [ value { "," value } ] "]" ;
record      = "{" [ pair { "," pair } ] "}" ;
pair        = IDENT ":" value ;
var_ref     = "$" IDENT { "." IDENT } ;
comment     = "#" TEXT EOL ;
```

Notes: a pipeline may be fed by a value (`$pages | md.render`); `var_ref`
supports field access (`$result.error.message`); `if` takes a single
comparison, not arbitrary expressions — v0.1 has **no loops** (see §10).

Deliberate exclusions, each closing a class of bugs:

| Excluded                  | Bug class it closes                          |
| ------------------------- | -------------------------------------------- |
| Unquoted barewords as strings | word-splitting / globbing surprises      |
| String interpolation in v0.1  | injection via interpolated values        |
| Aliases                   | "what does this actually run?"               |
| Implicit `$PATH` lookup of arbitrary binaries | running the wrong thing  |
| `eval` / dynamic code     | prompt-injection → code execution            |

Strings are always quoted (`"…"`). Variables are always `$name` and are passed
as **values**, never re-parsed as code. There is no glob syntax in v0.1;
globbing is a function: `glob "*.md"` returns a list of paths.

## 4. Commands are declared, not discovered

There is no `$PATH` scanning. Every callable command comes from a **manifest**
the runtime loads. A manifest entry:

```
command fs.remove {
  summary: "Delete files or directories"
  params: {
    paths:     { type: list<path>, required: true }
    recursive: { type: bool, default: false }
  }
  effects:    [write, destructive]
  idempotent: true          # deleting an absent path is not an error
  dry_run:    supported     # supported | unsupported | inherent
  output:     { removed: list<path> }
}
```

The manifest is machine-readable. An agent can list every command, its
parameter types, and its effects **without executing anything**. (This is the
same insight as MCP tool schemas, applied to a shell.)

External binaries are callable only through an explicit bridge:
`exec "ffmpeg" [args…]`, which is itself a command whose manifest declares
worst-case effects (`[read, write, net]`) — making "escaping to raw exec"
visible and auditable rather than the default.

## 5. Effects

The closed set for v0.1:

| Effect        | Meaning                                        |
| ------------- | ---------------------------------------------- |
| `read`        | reads files or system state                    |
| `write`       | creates or modifies files / state              |
| `net`         | any network access                             |
| `destructive` | irreversibly deletes or overwrites data        |

Rules:

1. A command may only perform effects it declares. The runtime sandboxes
   accordingly; a violation is a runtime error, not a warning.
2. A pipeline's effect set is the union of its commands' effects.
3. The runtime can be started with a policy, e.g.
   `cli run script.cli --deny destructive --confirm net` — the permission
   prompt problem becomes declarative.

## 6. Dry-run is runtime semantics

`cli run script.cli --dry-run` executes the script with every command in
dry-run mode:

- Commands with `dry_run: supported` return the envelope they *would* return,
  with `"simulated": true`, and perform no `write`/`destructive`/`net` effects.
- Commands with `dry_run: inherent` (pure reads) run normally.
- Commands with `dry_run: unsupported` return an error envelope — visible in
  the output, so a dry run tells you exactly which steps could not be
  simulated instead of silently skipping them.

## 7. Errors are data

The `Error` record:

```
{
  "code": "fs/not-found",        # namespaced, stable, documented
  "message": "No such path: /tmp/x",
  "retryable": false,
  "details": { "path": "/tmp/x" },
  "hint": "Run `glob \"/tmp/*\"` to inspect existing paths."
}
```

- `code` is stable API; `message` and `hint` are presentation and may change.
- `retryable` + the command's `idempotent` flag give agents a decidable retry
  policy: retry iff `retryable && idempotent`.
- `try <pipeline>` converts an error envelope into a value, so error handling
  is data flow, not control-flow acrobatics:

```
let result = try fs.remove --paths=["/tmp/build"] --recursive=true
if $result.status == "error" {
  log.warn --message=$result.error.message
}
```

## 8. Example

```
# Publish built docs: fail early, delete nothing without effects clearance.
let pages = glob "docs/*.md"

let built = $pages | md.render --theme="plain"

fs.write --dir="dist/" --files=$built

let result = try net.upload --to="s3://my-bucket/docs" --files=$built
if $result.status == "error" {
  log.error --message=$result.error.message
  exit --code=1
}
```

Running `cli run publish.cli --dry-run` simulates the write and upload and
reports what *would* happen. Running with `--deny net` fails the upload step at
the effects check — before any bytes leave the machine.

## 9. MCP interoperability

MCP is the established way agents reach tools. This project does not compete
with it — it **composes** it. Interop runs in both directions.

### 9.1 MCP tools as commands (client direction)

The runtime connects to MCP servers and synthesizes manifests from
`tools/list`: tool `create_issue` on server `github` becomes command
`mcp.github.create_issue`, params derived from the tool's JSON Schema. The
existing MCP ecosystem becomes the de-facto extended stdlib — nobody rewrites
anything to make their tools scriptable here.

Effect mapping from MCP tool annotations:

| MCP annotation           | Manifest field           |
| ------------------------ | ------------------------ |
| `readOnlyHint: true`     | `effects: [read]`        |
| `destructiveHint: true`  | `effects += destructive` |
| `openWorldHint: true`    | `effects += net`         |
| `idempotentHint`         | `idempotent`             |

Trust boundary — two rules:

1. MCP annotations are **hints from an untrusted server**. The runtime uses
   them for policy decisions at the call boundary (call / confirm / deny); it
   cannot sandbox what happens on the far side of the wire, and this spec
   must never claim otherwise.
2. A tool with **no annotations** gets worst-case effects
   (`[read, write, net, destructive]`) — exactly like the `exec` bridge:
   escaping the model is allowed, but visible and policy-gated.

Design constraint this imposes on §4: manifest parameter types are **JSON
Schema**, so `tools/list` maps 1:1 with no lossy translation.

### 9.2 The runtime as an MCP server (server direction)

`cli run` exposed as an MCP tool: any agent submits a script plus a policy
(`deny: [destructive]`) and gets the structured envelopes back. One
integration, every agent framework.

## 10. Open questions

1. **Loops** — v0.1 deliberately has `if` but no loops; iteration is expected
   to happen *inside* pipelines (commands over lists). Is that enough, or does
   a bounded `for` belong in the language?
2. **String interpolation** — excluded for safety, but painful. Is a
   non-executable template form (`fmt "hello {name}" --name=$n`) enough?
3. **Streaming** — envelopes are whole-value; large outputs need a chunked
   record stream design.
4. **Manifest distribution** — how are command manifests installed, versioned,
   and trusted? (Supply chain surface.)
5. **The name** — `.cli` is a placeholder; unsearchable as a project name.
