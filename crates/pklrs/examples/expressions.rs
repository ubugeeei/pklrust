/// Evaluate specific expressions within a module.
use pklrs::{EvaluatorManager, EvaluatorOptions, ModuleSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = EvaluatorManager::new()?;
    let evaluator = manager.new_evaluator(EvaluatorOptions::preconfigured())?;

    let source = ModuleSource::text(
        r#"
        name = "my-app"
        port = 8080
        url = "http://localhost:\(port)"
        hosts = new Listing { "web-1"; "web-2"; "web-3" }
        "#,
    );

    // Evaluate the whole module
    let full = manager.evaluate_module(&evaluator, source.clone())?;
    println!("Full module:\n{full:#?}\n");

    // Evaluate a single expression
    let name = manager.evaluate_expression(&evaluator, source.clone(), Some("name"))?;
    println!("name = {name:?}");

    let url = manager.evaluate_expression(&evaluator, source.clone(), Some("url"))?;
    println!("url = {url:?}");

    let hosts = manager.evaluate_expression(&evaluator, source, Some("hosts"))?;
    println!("hosts = {hosts:?}");

    manager.close_evaluator(&evaluator)?;
    Ok(())
}
