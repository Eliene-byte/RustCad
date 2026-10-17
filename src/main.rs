//! Ponto de entrada do RustCad — interface de linha de comando interativa.
//! Semelhante à linha de comando do AutoCAD.

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use rustcad::command::CommandInterpreter;
use rustcad::CadDocument;

fn main() -> Result<()> {
    let doc = CadDocument::new("Desenho sem nome");
    let mut interpreter = CommandInterpreter::new(doc);

    println!("========================================================");
    println!("  RustCad v0.1.0 — Clone do AutoCAD em Rust");
    println!("  Fase 1: 40% das funcionalidades do AutoCAD");
    println!("  Digite HELP para lista de comandos ou QUIT para sair.");
    println!("========================================================");

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()?;
    rl.load_history("rustcad_history.txt").ok();

    loop {
        let layer = interpreter
            .document()
            .layers
            .current_layer()
            .map(|l| l.name.as_str())
            .unwrap_or("0");
        let prompt = format!("[{}] Comando: ", layer);

        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    rl.add_history_entry(trimmed);
                }

                let upper = trimmed.to_uppercase();
                if upper == "QUIT" || upper == "EXIT" || upper == "Q" {
                    println!("{}", interpreter.execute(trimmed).message);
                    break;
                }

                let result = interpreter.execute(trimmed);
                if !result.message.is_empty() {
                    println!("{}", result.message);
                }

                if !result.success {
                    // continua mesmo com erro (não sai)
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("\nInterrompido. Digite QUIT para sair.");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("\nSaindo...");
                break;
            }
            Err(err) => {
                println!("Erro de leitura: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("rustcad_history.txt").ok();
    Ok(())
}
