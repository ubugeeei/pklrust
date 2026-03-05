use pklrs::{EvaluatorManager, EvaluatorOptions, ModuleSource};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Server {
    host: String,
    port: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = EvaluatorManager::new()?;
    let opts = EvaluatorOptions::preconfigured();
    let evaluator = manager.new_evaluator(opts)?;

    // Evaluate inline Pkl text
    let source = ModuleSource::text(
        r#"
        host = "localhost"
        port = 8080
        "#,
    );

    let server: Server = manager.evaluate_module_typed(&evaluator, source)?;
    println!("Server: {server:?}");

    // Evaluate a Pkl file (if it exists)
    // let source = ModuleSource::file("config.pkl");
    // let value = manager.evaluate_module(&evaluator, source)?;
    // println!("Value: {value:?}");

    manager.close_evaluator(&evaluator)?;
    Ok(())
}
