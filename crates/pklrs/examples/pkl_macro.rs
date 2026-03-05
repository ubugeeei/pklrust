/// Basic `pkl!` macro usage — write PKL as Rust tokens.
use pklrs::pkl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = pkl! {
        host = "localhost"
        port = 8080
        database {
            url = "postgres://localhost/mydb"
            maxConnections = 10
        }
    }?;

    println!("{value:#?}");
    Ok(())
}
