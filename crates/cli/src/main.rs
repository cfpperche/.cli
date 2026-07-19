use std::process::ExitCode;

use dotcli_parser::Statement;

const USAGE: &str = "\
usage: cli <subcommand>

subcommands:
  check [--ast] <file>...   parse and validate scripts (M1)
  run <file>                execute a script            (M2, not implemented)
  commands                  list available commands     (M3, not implemented)
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first().map(|(c, rest)| (c.as_str(), rest)) {
        Some(("check", rest)) => check(rest),
        Some(("run" | "commands", _)) => {
            eprintln!("not implemented yet — see ARCHITECTURE.md for the roadmap");
            ExitCode::from(2)
        }
        Some(("help" | "--help" | "-h", _)) => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn check(rest: &[String]) -> ExitCode {
    let mut show_ast = false;
    let mut files: Vec<&String> = Vec::new();
    for arg in rest {
        if arg == "--ast" {
            show_ast = true;
        } else {
            files.push(arg);
        }
    }
    if files.is_empty() {
        eprintln!("usage: cli check [--ast] <file>...");
        return ExitCode::from(2);
    }

    let mut failed = false;
    for file in files {
        let src = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{file}: {e}");
                failed = true;
                continue;
            }
        };
        match dotcli_parser::parse(&src) {
            Ok(stmts) => {
                println!(
                    "{file}: OK ({} statements, {} commands)",
                    stmts.len(),
                    count_commands(&stmts)
                );
                if show_ast {
                    println!("{stmts:#?}");
                }
            }
            Err(e) => {
                // `e` renders as `line:col: message`.
                eprintln!("{file}:{e}");
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn count_commands(stmts: &[Statement]) -> usize {
    stmts
        .iter()
        .map(|s| match s {
            Statement::Binding { pipeline, .. } => pipeline.commands.len(),
            Statement::Pipeline(p) => p.commands.len(),
            Statement::If {
                then_block,
                else_block,
                ..
            } => count_commands(then_block) + else_block.as_deref().map_or(0, count_commands),
        })
        .sum()
}
