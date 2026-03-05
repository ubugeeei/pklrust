/// Working with raw PklValue without serde deserialization.
///
/// Useful when you need dynamic access to PKL data or want to
/// inspect the structure before deserializing.
use pklrs::{EvaluatorManager, EvaluatorOptions, ModuleSource, PklValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = EvaluatorManager::new()?;
    let evaluator = manager.new_evaluator(EvaluatorOptions::preconfigured())?;

    let source = ModuleSource::text(
        r#"
        name = "example"
        port = 8080
        debug = true
        tags = new Listing { "web"; "api" }
        "#,
    );

    let value = manager.evaluate_module(&evaluator, source)?;

    // Access properties via the convenience method
    if let Some(props) = value.as_properties() {
        for (key, val) in &props {
            match val {
                PklValue::String(s) => println!("{key} = \"{s}\""),
                PklValue::Int(n) => println!("{key} = {n}"),
                PklValue::Bool(b) => println!("{key} = {b}"),
                PklValue::List(items) => {
                    let strs: Vec<_> = items
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect();
                    println!("{key} = {strs:?}");
                }
                other => println!("{key} = {other:?}"),
            }
        }
    }

    // Direct value accessors
    let name = value
        .as_properties()
        .and_then(|p| p.get("name").copied())
        .and_then(|v| v.as_str());
    println!("\nDirect access — name: {name:?}");

    manager.close_evaluator(&evaluator)?;
    Ok(())
}
