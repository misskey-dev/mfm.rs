use std::time::Instant;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    println!("interactive parser");
    println!("Ctrl+D to exit.");
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line)?;
                let line = line
                    .replace(r"\n", "\n")
                    .replace(r"\r", "\r")
                    .replace(r"\u00a0", "\u{00a0}");
                let start = Instant::now();
                let result = mfm::parse(&line);
                let end = start.elapsed();
                match result {
                    Ok(nodes) => {
                        println!("{nodes:?}");
                        println!("parsing time: {:.3}ms", end.as_micros() as f64 / 1000.0)
                    }
                    Err(err) => eprintln!("Error: {err:?}"),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted.");
            }
            Err(ReadlineError::Eof) => {
                println!("Bye.");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }
    Ok(())
}
