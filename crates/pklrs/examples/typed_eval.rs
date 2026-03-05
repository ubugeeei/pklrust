/// Typed evaluation — deserialize PKL into Rust structs via serde.
use pklrs::{EvaluatorManager, EvaluatorOptions, ModuleSource};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    name: String,
    version: String,
    server: ServerConfig,
    features: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    #[serde(rename = "maxConnections")]
    max_connections: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = EvaluatorManager::new()?;
    let evaluator = manager.new_evaluator(EvaluatorOptions::preconfigured())?;

    let source = ModuleSource::text(
        r#"
        name = "my-service"
        version = "2.1.0"
        server {
            host = "0.0.0.0"
            port = 8443
            maxConnections = 100
        }
        features = new Listing {
            "auth"
            "logging"
            "metrics"
        }
        "#,
    );

    let config: AppConfig = manager.evaluate_module_typed(&evaluator, source)?;
    println!("App: {} v{}", config.name, config.version);
    println!("Server: {}:{}", config.server.host, config.server.port);
    println!("Max connections: {}", config.server.max_connections);
    println!("Features: {:?}", config.features);

    manager.close_evaluator(&evaluator)?;
    Ok(())
}
