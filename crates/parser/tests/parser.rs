use dotcli_parser::{parse, Argument, CompareOp, Statement, Value};

/// The shipped examples are also parser fixtures: if they stop parsing,
/// either the spec or the examples changed without the other.
#[test]
fn parses_example_publish() {
    let src = include_str!("../../../examples/publish.cli");
    let stmts = parse(src).expect("examples/publish.cli must parse");
    assert_eq!(stmts.len(), 6);
}

#[test]
fn parses_example_cleanup() {
    let src = include_str!("../../../examples/cleanup.cli");
    let stmts = parse(src).expect("examples/cleanup.cli must parse");
    assert_eq!(stmts.len(), 2);
}

#[test]
fn binding_with_flags_and_list() {
    let stmts = parse(r#"let r = fs.remove --paths=["/tmp/a", "/tmp/b"] --recursive=true"#).unwrap();
    let Statement::Binding { name, pipeline } = &stmts[0] else { panic!("expected binding") };
    assert_eq!(name, "r");
    assert!(!pipeline.tried);
    let cmd = &pipeline.commands[0];
    assert_eq!(cmd.name, "fs.remove");
    assert_eq!(
        cmd.args[0],
        Argument::Named {
            name: "paths".into(),
            value: Some(Value::List(vec![
                Value::Str("/tmp/a".into()),
                Value::Str("/tmp/b".into()),
            ])),
        }
    );
    assert_eq!(
        cmd.args[1],
        Argument::Named { name: "recursive".into(), value: Some(Value::Bool(true)) }
    );
}

#[test]
fn try_sets_tried() {
    let stmts = parse("let r = try net.upload --to=\"s3://b\"").unwrap();
    let Statement::Binding { pipeline, .. } = &stmts[0] else { panic!() };
    assert!(pipeline.tried);
}

#[test]
fn value_fed_pipeline() {
    let stmts = parse("$pages | md.render --theme=\"plain\" | fs.write --dir=\"dist/\"").unwrap();
    let Statement::Pipeline(p) = &stmts[0] else { panic!() };
    assert_eq!(p.source, Some(Value::Var(vec!["pages".into()])));
    assert_eq!(p.commands.len(), 2);
    assert_eq!(p.commands[0].name, "md.render");
}

#[test]
fn var_path_is_split() {
    let stmts = parse("log.error --message=$result.error.message").unwrap();
    let Statement::Pipeline(p) = &stmts[0] else { panic!() };
    let Argument::Named { value: Some(Value::Var(path)), .. } = &p.commands[0].args[0] else {
        panic!()
    };
    assert_eq!(path, &["result", "error", "message"]);
}

#[test]
fn if_else_with_comparison() {
    let src = r#"
if $r.status == "error" {
  log.warn --message="bad"
} else {
  log.info --message="good"
}
"#;
    let stmts = parse(src).unwrap();
    let Statement::If { condition, then_block, else_block } = &stmts[0] else { panic!() };
    assert_eq!(condition.left, Value::Var(vec!["r".into(), "status".into()]));
    assert_eq!(condition.comparison, Some((CompareOp::Eq, Value::Str("error".into()))));
    assert_eq!(then_block.len(), 1);
    assert_eq!(else_block.as_ref().unwrap().len(), 1);
}

#[test]
fn pipe_continues_across_lines() {
    let stmts = parse("glob \"*.md\" |\n  md.render").unwrap();
    let Statement::Pipeline(p) = &stmts[0] else { panic!() };
    assert_eq!(p.commands.len(), 2);
}

#[test]
fn record_value_and_bare_flag() {
    let stmts = parse("http.get --headers={accept: \"text/html\", retries: 3} --verbose").unwrap();
    let Statement::Pipeline(p) = &stmts[0] else { panic!() };
    assert_eq!(
        p.commands[0].args[1],
        Argument::Named { name: "verbose".into(), value: None }
    );
}

#[test]
fn bareword_is_rejected_with_hint() {
    let err = parse("fs.remove foo").unwrap_err();
    assert!(err.message.contains("bareword"), "got: {}", err.message);
    assert!(err.message.contains("\"foo\""), "hint must show the fix: {}", err.message);
}

#[test]
fn unterminated_string_is_an_error() {
    let err = parse("log.info --message=\"oops").unwrap_err();
    assert!(err.message.contains("unterminated string"), "got: {}", err.message);
}

#[test]
fn lone_value_statement_is_rejected() {
    let err = parse("$x").unwrap_err();
    assert!(err.message.contains("pipe it into a command"), "got: {}", err.message);
}

#[test]
fn negative_and_float_numbers() {
    let stmts = parse("calc.add -1 2.5").unwrap();
    let Statement::Pipeline(p) = &stmts[0] else { panic!() };
    assert_eq!(p.commands[0].args[0], Argument::Positional(Value::Number(-1.0)));
    assert_eq!(p.commands[0].args[1], Argument::Positional(Value::Number(2.5)));
}

#[test]
fn comments_and_blank_lines_are_skipped() {
    let src = "# a comment\n\n# another\nlog.info --message=\"hi\"\n";
    let stmts = parse(src).unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn unquoted_path_gets_a_quoting_hint() {
    let err = parse("fs.remove /tmp/foo").unwrap_err();
    assert!(err.message.contains("must be quoted"), "got: {}", err.message);
}

#[test]
fn glob_char_points_to_the_glob_command() {
    let err = parse("fs.remove *.log").unwrap_err();
    assert!(err.message.contains("glob command"), "got: {}", err.message);
}

#[test]
fn error_positions_are_one_based() {
    let err = parse("log.info !\n").unwrap_err();
    assert_eq!((err.line, err.col), (1, 10));
}
