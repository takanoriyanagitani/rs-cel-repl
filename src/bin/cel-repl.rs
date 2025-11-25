use cel::Context;
use clap::{ArgGroup, Parser};
use rs_cel_repl::{CelValue, Error, compile};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, BufWriter, Write};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(group(
    ArgGroup::new("context_group")
        .args(&["context_file", "json_context"]),
))]
struct Cli {
    /// The CEL expression to evaluate
    #[arg(short, long = "expression")]
    expr: Option<String>,

    /// Path to a JSON file with context variables
    #[arg(short, long = "context-file")]
    context_file: Option<String>,

    /// JSON string with context variables
    #[arg(long = "json-context")]
    json_context: Option<String>,

    /// Name of the variable representing the stdin input in CEL expressions
    #[arg(long = "input-variable", default_value = "input")]
    input_variable: String,
}

fn create_context(
    context_file: Option<String>,
    json_context: Option<String>,
) -> Result<Context<'static>, Error> {
    if let Some(path) = context_file {
        let content = fs::read_to_string(&path)?;
        let json: Value = serde_json::from_str(&content)?;

        let mut ctx = Context::default();
        if let Value::Object(map) = json {
            for (key, value) in map {
                ctx.add_variable(&key, value)
                    .map_err(|e| Error::Cel(e.to_string()))?;
            }
        }
        Ok(ctx)
    } else if let Some(json_str) = json_context {
        let json: Value = serde_json::from_str(&json_str)?;

        let mut ctx = Context::default();
        if let Value::Object(map) = json {
            for (key, value) in map {
                ctx.add_variable(&key, value)
                    .map_err(|e| Error::Cel(e.to_string()))?;
            }
        }
        Ok(ctx)
    } else {
        Ok(Context::default())
    }
}

fn run_direct(
    expr: &str,
    context_file: Option<String>,
    json_context: Option<String>,
    input_variable: &str,
) -> Result<(), Error> {
    let ctx = create_context(context_file, json_context)?;

    let prog = compile(expr)?;

    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    for line in reader.lines() {
        let line = line?;
        let input_data: Value = serde_json::from_str(&line).unwrap_or(Value::String(line));

        let result = prog.execute_with_value(&ctx, input_variable, input_data)?;
        let result_json: Value = CelValue(result).into();
        serde_json::to_writer(&mut writer, &result_json)?;
        writeln!(&mut writer)?;
    }

    Ok(())
}

fn run_repl(context_file: Option<String>, json_context: Option<String>) -> Result<(), Error> {
    let ctx = create_context(context_file, json_context)?;

    let mut rl = DefaultEditor::new()?;
    println!("CEL REPL - type '.exit' or Ctrl+D to quit");

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line == ".exit" {
                    break;
                }
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line)?;

                match compile(line) {
                    Ok(prog) => match prog.execute(&ctx) {
                        Ok(result) => {
                            let result_json: Value = CelValue(result).into();
                            println!("{}", serde_json::to_string_pretty(&result_json)?);
                        }
                        Err(e) => eprintln!("Evaluation error: {}", e),
                    },
                    Err(e) => eprintln!("Compilation error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C received. Exiting REPL.");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D received. Exiting REPL.");
                break;
            }
            Err(err) => {
                eprintln!("Error reading line: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    if let Some(expr) = cli.expr {
        run_direct(
            &expr,
            cli.context_file,
            cli.json_context,
            &cli.input_variable,
        )?;
    } else {
        run_repl(cli.context_file, cli.json_context)?;
    }

    Ok(())
}
