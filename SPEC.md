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

Non-goals (for now): interactive ergonomics (completions, prompts), job
control, POSIX compatibility, performance.

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
script     = { statement } ;
statement  = pipeline | binding | comment ;
binding    = "let" IDENT "=" pipeline ;
pipeline   = command { "|" command } ;
command    = IDENT { argument } ;
argument   = named_arg | value ;
named_arg  = "--" IDENT [ "=" value ] ;
value      = STRING | NUMBER | BOOL | list | record | var_ref ;
list       = "[" [ value { "," value } ] "]" ;
record     = "{" [ pair { "," pair } ] "}" ;
pair       = IDENT ":" value ;
var_ref    = "$" IDENT ;
comment    = "#" TEXT EOL ;
```

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

## 9. Open questions

1. **Conditionals/loops** — §7 sneaks in `if`; how much control flow belongs in
   v0.1 vs. delegating to a host language?
2. **String interpolation** — excluded for safety, but painful. Is a
   non-executable template form (`fmt "hello {name}" --name=$n`) enough?
3. **Streaming** — envelopes are whole-value; large outputs need a chunked
   record stream design.
4. **Manifest distribution** — how are command manifests installed, versioned,
   and trusted? (Supply chain surface.)
5. **The name** — `.cli` is a placeholder; unsearchable as a project name.
