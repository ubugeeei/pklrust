/// Advanced `pkl!` macro usage — classes, functions, control flow, units.
use pklrs::pkl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Duration and data size units
    let value = pkl! {
        timeout = 30.s
        retryDelay = 500.ms
        ttl = 1.h
        maxUpload = 512.mb
    }?;
    println!("Units:\n{value:#?}\n");

    // Functions and local variables
    let value = pkl! {
        local basePort = 8000
        function portFor(offset) = basePort + offset

        http = portFor(80)
        https = portFor(443)
    }?;
    println!("Functions:\n{value:#?}\n");

    // Conditional expressions
    let value = pkl! {
        local debug = true
        port = if (debug) 3000 else 8080
        logLevel = if (debug) "debug" else "info"
    }?;
    println!("Conditionals:\n{value:#?}\n");

    // Nested objects with type annotations
    let value = pkl! {
        server {
            host: String = "0.0.0.0"
            port: UInt16 = 443
        }
        database {
            host = "db.internal"
            port = 5432
            maxConnections = 20
        }
    }?;
    println!("Nested:\n{value:#?}\n");

    // Listings
    let value = pkl! {
        environments = new Listing {
            "development"
            "staging"
            "production"
        }
    }?;
    println!("Listing:\n{value:#?}\n");

    // Mapping
    let value = pkl! {
        ports = new Mapping {
            ["http"] = 80
            ["https"] = 443
            ["ssh"] = 22
        }
    }?;
    println!("Mapping:\n{value:#?}\n");

    Ok(())
}
