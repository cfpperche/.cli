/// One top-level (or block-level) statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// `let name = pipeline`
    Binding { name: String, pipeline: Pipeline },
    /// A bare pipeline evaluated for its effects/output.
    Pipeline(Pipeline),
    /// `if <condition> { … } [else { … } | else if …]`
    If {
        condition: Condition,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
}

/// `value` or `value == value` / `value != value`.
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub left: Value,
    pub comparison: Option<(CompareOp, Value)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    NotEq,
}

/// `[try] (command | value) { | command }`
#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    /// Prefixed with `try`: an error envelope becomes a value.
    pub tried: bool,
    /// A value feeding the pipe (`$pages | md.render`), if any.
    pub source: Option<Value>,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Possibly dotted: `fs.remove`, `glob`.
    pub name: String,
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// `--name` or `--name=value`. A bare flag carries no value.
    Named {
        name: String,
        value: Option<Value>,
    },
    Positional(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    List(Vec<Value>),
    Record(Vec<(String, Value)>),
    /// `$result.error.message` → `["result", "error", "message"]`
    Var(Vec<String>),
}
