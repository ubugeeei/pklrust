/// Evaluate multiple modules with the same evaluator.
///
/// Shows how to reuse a single evaluator for multiple evaluations,
/// which is more efficient than creating a new one each time.
use pklrs::{EvaluatorManager, EvaluatorOptions, ModuleSource};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DbConfig {
    host: String,
    port: u16,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CacheConfig {
    host: String,
    port: u16,
    ttl: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = EvaluatorManager::new()?;
    let evaluator = manager.new_evaluator(EvaluatorOptions::preconfigured())?;

    // First module: database config
    let db: DbConfig = manager.evaluate_module_typed(
        &evaluator,
        ModuleSource::text(
            r#"
            host = "db.internal"
            port = 5432
            name = "myapp_production"
            "#,
        ),
    )?;
    println!("Database: {db:?}");

    // Second module: cache config
    let cache: CacheConfig = manager.evaluate_module_typed(
        &evaluator,
        ModuleSource::text(
            r#"
            host = "redis.internal"
            port = 6379
            ttl = 3600
            "#,
        ),
    )?;
    println!("Cache: {cache:?}");

    manager.close_evaluator(&evaluator)?;
    Ok(())
}
