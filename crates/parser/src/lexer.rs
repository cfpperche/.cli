use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Possibly dotted: `glob`, `fs.remove`, `dry-run`.
    Ident(String),
    Str(String),
    Number(f64),
    Bool(bool),
    /// `$result.error.message` → path segments.
    Var(Vec<String>),
    /// `--recursive` → `recursive`.
    Flag(String),
    Let,
    If,
    Else,
    Try,
    Pipe,
    /// `=`
    Assign,
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Newline,
}

impl Token {
    /// Short human/agent-readable description for diagnostics.
    pub fn describe(&self) -> String {
        match self {
            Token::Ident(s) => format!("identifier \"{s}\""),
            Token::Str(_) => "string".into(),
            Token::Number(n) => format!("number {n}"),
            Token::Bool(b) => format!("boolean {b}"),
            Token::Var(path) => format!("variable ${}", path.join(".")),
            Token::Flag(f) => format!("flag --{f}"),
            Token::Let => "keyword `let`".into(),
            Token::If => "keyword `if`".into(),
            Token::Else => "keyword `else`".into(),
            Token::Try => "keyword `try`".into(),
            Token::Pipe => "`|`".into(),
            Token::Assign => "`=`".into(),
            Token::EqEq => "`==`".into(),
            Token::NotEq => "`!=`".into(),
            Token::LBracket => "`[`".into(),
            Token::RBracket => "`]`".into(),
            Token::LBrace => "`{`".into(),
            Token::RBrace => "`}`".into(),
            Token::Comma => "`,`".into(),
            Token::Colon => "`:`".into(),
            Token::Newline => "end of line".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
}

pub fn lex(src: &str) -> Result<Vec<Spanned>, ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens: Vec<Spanned> = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;

    macro_rules! push {
        ($tok:expr, $line:expr, $col:expr) => {
            tokens.push(Spanned { token: $tok, line: $line, col: $col })
        };
    }

    while i < chars.len() {
        let c = chars[i];
        let (tline, tcol) = (line, col);
        match c {
            ' ' | '\t' | '\r' => {
                i += 1;
                col += 1;
            }
            '\n' => {
                push!(Token::Newline, tline, tcol);
                i += 1;
                line += 1;
                col = 1;
            }
            '#' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                // The '\n' (if any) is handled on the next iteration.
            }
            '"' => {
                i += 1;
                col += 1;
                let mut s = String::new();
                let mut closed = false;
                while i < chars.len() {
                    let ch = chars[i];
                    match ch {
                        '"' => {
                            closed = true;
                            i += 1;
                            col += 1;
                            break;
                        }
                        '\n' => {
                            return Err(ParseError::new(
                                tline,
                                tcol,
                                "unterminated string — strings may not span lines",
                            ));
                        }
                        '\\' => {
                            i += 1;
                            col += 1;
                            let esc = match chars.get(i) {
                                Some(e) => *e,
                                None => break,
                            };
                            let real = match esc {
                                'n' => '\n',
                                't' => '\t',
                                '"' => '"',
                                '\\' => '\\',
                                other => {
                                    return Err(ParseError::new(
                                        line,
                                        col,
                                        format!(
                                            "unknown escape sequence \\{other} — valid: \\n \\t \\\" \\\\"
                                        ),
                                    ));
                                }
                            };
                            s.push(real);
                            i += 1;
                            col += 1;
                        }
                        _ => {
                            s.push(ch);
                            i += 1;
                            col += 1;
                        }
                    }
                }
                if !closed {
                    return Err(ParseError::new(tline, tcol, "unterminated string"));
                }
                push!(Token::Str(s), tline, tcol);
            }
            '|' => {
                push!(Token::Pipe, tline, tcol);
                i += 1;
                col += 1;
            }
            '[' => {
                push!(Token::LBracket, tline, tcol);
                i += 1;
                col += 1;
            }
            ']' => {
                push!(Token::RBracket, tline, tcol);
                i += 1;
                col += 1;
            }
            '{' => {
                push!(Token::LBrace, tline, tcol);
                i += 1;
                col += 1;
            }
            '}' => {
                push!(Token::RBrace, tline, tcol);
                i += 1;
                col += 1;
            }
            ',' => {
                push!(Token::Comma, tline, tcol);
                i += 1;
                col += 1;
            }
            ':' => {
                push!(Token::Colon, tline, tcol);
                i += 1;
                col += 1;
            }
            '=' => {
                if chars.get(i + 1) == Some(&'=') {
                    push!(Token::EqEq, tline, tcol);
                    i += 2;
                    col += 2;
                } else {
                    push!(Token::Assign, tline, tcol);
                    i += 1;
                    col += 1;
                }
            }
            '!' => {
                if chars.get(i + 1) == Some(&'=') {
                    push!(Token::NotEq, tline, tcol);
                    i += 2;
                    col += 2;
                } else {
                    return Err(ParseError::new(tline, tcol, "unexpected `!` — did you mean `!=`?"));
                }
            }
            '-' => {
                if chars.get(i + 1) == Some(&'-') {
                    i += 2;
                    col += 2;
                    let start = i;
                    while i < chars.len() && is_ident_continue(chars[i]) {
                        i += 1;
                        col += 1;
                    }
                    if start == i {
                        return Err(ParseError::new(
                            tline,
                            tcol,
                            "expected a flag name after `--` (e.g. --recursive)",
                        ));
                    }
                    let name: String = chars[start..i].iter().collect();
                    push!(Token::Flag(name), tline, tcol);
                } else if chars.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
                    let (tok, len) = lex_number(&chars[i..], tline, tcol)?;
                    push!(tok, tline, tcol);
                    i += len;
                    col += len;
                } else {
                    return Err(ParseError::new(
                        tline,
                        tcol,
                        "unexpected `-` — flags are `--name`, negative numbers are `-1`",
                    ));
                }
            }
            '$' => {
                i += 1;
                col += 1;
                let start = i;
                while i < chars.len() && is_ident_continue(chars[i]) {
                    i += 1;
                    col += 1;
                }
                if start == i {
                    return Err(ParseError::new(
                        tline,
                        tcol,
                        "expected a variable name after `$` (e.g. $result)",
                    ));
                }
                let raw: String = chars[start..i].iter().collect();
                let path: Vec<String> = raw.split('.').map(str::to_string).collect();
                if path.iter().any(String::is_empty) {
                    return Err(ParseError::new(
                        tline,
                        tcol,
                        format!("malformed variable path ${raw} — empty segment"),
                    ));
                }
                push!(Token::Var(path), tline, tcol);
            }
            c if c.is_ascii_digit() => {
                let (tok, len) = lex_number(&chars[i..], tline, tcol)?;
                push!(tok, tline, tcol);
                i += len;
                col += len;
            }
            c if is_ident_start(c) => {
                let start = i;
                while i < chars.len() && is_ident_continue(chars[i]) {
                    i += 1;
                    col += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let tok = match word.as_str() {
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "try" => Token::Try,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Ident(word),
                };
                push!(tok, tline, tcol);
            }
            '/' | '~' => {
                return Err(ParseError::new(
                    tline,
                    tcol,
                    format!("unexpected `{c}` — paths are strings and must be quoted (e.g. \"/tmp/build\")"),
                ));
            }
            '*' => {
                return Err(ParseError::new(
                    tline,
                    tcol,
                    "unexpected `*` — there is no glob syntax; use the glob command (glob \"*.md\")",
                ));
            }
            other => {
                return Err(ParseError::new(
                    tline,
                    tcol,
                    format!("unexpected character `{other}`"),
                ));
            }
        }
    }

    Ok(tokens)
}

/// Lex a number starting at `chars[0]` (which may be `-`). Returns the token
/// and how many chars were consumed.
fn lex_number(chars: &[char], line: usize, col: usize) -> Result<(Token, usize), ParseError> {
    let mut len = 0;
    if chars.first() == Some(&'-') {
        len += 1;
    }
    while chars.get(len).is_some_and(|c| c.is_ascii_digit()) {
        len += 1;
    }
    if chars.get(len) == Some(&'.') && chars.get(len + 1).is_some_and(|c| c.is_ascii_digit()) {
        len += 1;
        while chars.get(len).is_some_and(|c| c.is_ascii_digit()) {
            len += 1;
        }
    }
    let raw: String = chars[..len].iter().collect();
    match raw.parse::<f64>() {
        Ok(n) => Ok((Token::Number(n), len)),
        Err(_) => Err(ParseError::new(line, col, format!("malformed number `{raw}`"))),
    }
}
