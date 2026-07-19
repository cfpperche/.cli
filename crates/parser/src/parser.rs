use crate::ast::*;
use crate::error::ParseError;
use crate::lexer::{lex, Spanned, Token};

pub fn parse(src: &str) -> Result<Vec<Statement>, ParseError> {
    let tokens = lex(src)?;
    let mut p = Parser { tokens, pos: 0 };
    p.parse_script()
}

struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|s| &s.token)
    }

    fn advance(&mut self) -> Option<Spanned> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(Token::Newline)) {
            self.pos += 1;
        }
    }

    /// Position of the current token (or the last one, at EOF) for errors.
    fn here(&self) -> (usize, usize) {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .map(|s| (s.line, s.col))
            .unwrap_or((1, 1))
    }

    fn err(&self, message: impl Into<String>) -> ParseError {
        let (line, col) = self.here();
        ParseError::new(line, col, message)
    }

    fn found(&self) -> String {
        match self.peek() {
            Some(t) => t.describe(),
            None => "end of file".into(),
        }
    }

    fn expect_ident(&mut self, what: &str) -> Result<String, ParseError> {
        match self.peek() {
            Some(Token::Ident(_)) => {
                let Some(Spanned { token: Token::Ident(s), .. }) = self.advance() else {
                    unreachable!()
                };
                Ok(s)
            }
            _ => Err(self.err(format!("expected {what}, found {}", self.found()))),
        }
    }

    fn expect(&mut self, tok: Token, msg: &str) -> Result<(), ParseError> {
        if self.peek() == Some(&tok) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(format!("{msg}, found {}", self.found())))
        }
    }

    fn parse_script(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek().is_none() {
                return Ok(stmts);
            }
            stmts.push(self.parse_statement()?);
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            Some(Token::Let) => {
                self.pos += 1;
                let name = self.expect_ident("a binding name after `let`")?;
                self.expect(Token::Assign, "expected `=` after the binding name")?;
                let pipeline = self.parse_pipeline()?;
                self.end_of_statement()?;
                Ok(Statement::Binding { name, pipeline })
            }
            Some(Token::If) => self.parse_if(),
            _ => {
                let pipeline = self.parse_pipeline()?;
                self.end_of_statement()?;
                Ok(Statement::Pipeline(pipeline))
            }
        }
    }

    /// A statement ends at a newline, at EOF, or just before the `}` that
    /// closes the surrounding block (left for the block parser to consume).
    fn end_of_statement(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            None | Some(Token::RBrace) => Ok(()),
            Some(Token::Newline) => {
                self.pos += 1;
                Ok(())
            }
            Some(Token::Ident(s)) => {
                let s = s.clone();
                Err(self.err(format!(
                    "unexpected bareword `{s}` — strings must be quoted (\"{s}\")"
                )))
            }
            Some(t) => Err(self.err(format!("expected end of statement, found {}", t.describe()))),
        }
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::If, "expected `if`")?;
        let left = self.parse_value()?;
        let comparison = match self.peek() {
            Some(Token::EqEq) => {
                self.pos += 1;
                Some((CompareOp::Eq, self.parse_value()?))
            }
            Some(Token::NotEq) => {
                self.pos += 1;
                Some((CompareOp::NotEq, self.parse_value()?))
            }
            _ => None,
        };
        self.expect(Token::LBrace, "expected `{` to open the if block")?;
        let then_block = self.parse_block()?;
        let else_block = if self.peek() == Some(&Token::Else) {
            self.pos += 1;
            if self.peek() == Some(&Token::If) {
                // `else if …` — represent the chained if as the sole
                // statement of the else block.
                Some(vec![self.parse_if()?])
            } else {
                self.expect(Token::LBrace, "expected `{` (or `if`) after `else`")?;
                Some(self.parse_block()?)
            }
        } else {
            self.end_of_statement()?;
            None
        };
        Ok(Statement::If { condition: Condition { left, comparison }, then_block, else_block })
    }

    /// Parses statements until the closing `}` (consumed). The opening `{`
    /// must already be consumed.
    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                None => return Err(self.err("unclosed block — expected `}`")),
                Some(Token::RBrace) => {
                    self.pos += 1;
                    return Ok(stmts);
                }
                _ => stmts.push(self.parse_statement()?),
            }
        }
    }

    fn parse_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        let tried = if self.peek() == Some(&Token::Try) {
            self.pos += 1;
            true
        } else {
            false
        };

        let mut source = None;
        let mut commands = Vec::new();
        if self.peek().is_some_and(starts_value) {
            source = Some(self.parse_value()?);
        } else {
            commands.push(self.parse_command()?);
        }

        while self.peek() == Some(&Token::Pipe) {
            self.pos += 1;
            // A trailing `|` continues the pipeline on the next line.
            self.skip_newlines();
            commands.push(self.parse_command()?);
        }

        if commands.is_empty() {
            return Err(self.err(
                "a value on its own is not a statement — pipe it into a command (`$x | cmd`)",
            ));
        }
        Ok(Pipeline { tried, source, commands })
    }

    fn parse_command(&mut self) -> Result<Command, ParseError> {
        let name = self.expect_ident("a command name")?;
        let mut args = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Flag(_)) => {
                    let Some(Spanned { token: Token::Flag(flag), .. }) = self.advance() else {
                        unreachable!()
                    };
                    let value = if self.peek() == Some(&Token::Assign) {
                        self.pos += 1;
                        Some(self.parse_value()?)
                    } else {
                        None
                    };
                    args.push(Argument::Named { name: flag, value });
                }
                Some(t) if starts_value(t) => {
                    args.push(Argument::Positional(self.parse_value()?));
                }
                _ => return Ok(Command { name, args }),
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.peek() {
            Some(Token::Str(_)) => {
                let Some(Spanned { token: Token::Str(s), .. }) = self.advance() else {
                    unreachable!()
                };
                Ok(Value::Str(s))
            }
            Some(Token::Number(_)) => {
                let Some(Spanned { token: Token::Number(n), .. }) = self.advance() else {
                    unreachable!()
                };
                Ok(Value::Number(n))
            }
            Some(Token::Bool(_)) => {
                let Some(Spanned { token: Token::Bool(b), .. }) = self.advance() else {
                    unreachable!()
                };
                Ok(Value::Bool(b))
            }
            Some(Token::Var(_)) => {
                let Some(Spanned { token: Token::Var(path), .. }) = self.advance() else {
                    unreachable!()
                };
                Ok(Value::Var(path))
            }
            Some(Token::LBracket) => self.parse_list(),
            Some(Token::LBrace) => self.parse_record(),
            Some(Token::Ident(s)) => {
                let s = s.clone();
                Err(self.err(format!(
                    "unexpected bareword `{s}` — strings must be quoted (\"{s}\")"
                )))
            }
            _ => Err(self.err(format!("expected a value, found {}", self.found()))),
        }
    }

    fn parse_list(&mut self) -> Result<Value, ParseError> {
        self.expect(Token::LBracket, "expected `[`")?;
        let mut items = Vec::new();
        self.skip_newlines();
        if self.peek() == Some(&Token::RBracket) {
            self.pos += 1;
            return Ok(Value::List(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_newlines();
            match self.peek() {
                Some(Token::Comma) => {
                    self.pos += 1;
                    self.skip_newlines();
                    // Trailing comma before `]` is allowed.
                    if self.peek() == Some(&Token::RBracket) {
                        self.pos += 1;
                        return Ok(Value::List(items));
                    }
                }
                Some(Token::RBracket) => {
                    self.pos += 1;
                    return Ok(Value::List(items));
                }
                _ => {
                    return Err(self.err(format!(
                        "expected `,` or `]` in list, found {}",
                        self.found()
                    )));
                }
            }
        }
    }

    fn parse_record(&mut self) -> Result<Value, ParseError> {
        self.expect(Token::LBrace, "expected `{`")?;
        let mut pairs = Vec::new();
        self.skip_newlines();
        if self.peek() == Some(&Token::RBrace) {
            self.pos += 1;
            return Ok(Value::Record(pairs));
        }
        loop {
            let key = self.expect_ident("a record key")?;
            self.expect(Token::Colon, "expected `:` after the record key")?;
            self.skip_newlines();
            let value = self.parse_value()?;
            pairs.push((key, value));
            self.skip_newlines();
            match self.peek() {
                Some(Token::Comma) => {
                    self.pos += 1;
                    self.skip_newlines();
                    if self.peek() == Some(&Token::RBrace) {
                        self.pos += 1;
                        return Ok(Value::Record(pairs));
                    }
                }
                Some(Token::RBrace) => {
                    self.pos += 1;
                    return Ok(Value::Record(pairs));
                }
                _ => {
                    return Err(self.err(format!(
                        "expected `,` or `}}` in record, found {}",
                        self.found()
                    )));
                }
            }
        }
    }
}

fn starts_value(t: &Token) -> bool {
    matches!(
        t,
        Token::Str(_)
            | Token::Number(_)
            | Token::Bool(_)
            | Token::Var(_)
            | Token::LBracket
            | Token::LBrace
    )
}
