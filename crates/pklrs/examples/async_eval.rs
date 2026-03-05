/// Async evaluation example.
///
/// This example demonstrates asynchronous evaluation using tokio.
/// Build with: `cargo run --example async_eval --features async`
///
/// Note: The async evaluator API is currently in development.
/// For now, the sync API can be used with `tokio::task::spawn_blocking`.
use pklrs::{EvaluatorManager, EvaluatorOptions, ModuleSource};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    name: String,
    version: String,
    debug: bool,
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = tokio::task::spawn_blocking(|| {
        let mut manager = EvaluatorManager::new()?;
        let opts = EvaluatorOptions::preconfigured();
        let evaluator = manager.new_evaluator(opts)?;

        let source = ModuleSource::text(
            r#"
            name = "my-app"
            version = "1.0.0"
            debug = true
            "#,
        );

        let result: Config = manager.evaluate_module_typed(&evaluator, source)?;
        manager.close_evaluator(&evaluator)?;
        Ok::<_, pklrs::Error>(result)
    })
    .await??;

    println!("Config: {config:?}");
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature.");
    eprintln!("Run with: cargo run --example async_eval --features async");
}
