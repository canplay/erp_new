//! Build script for billing-service

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell cargo to recompile if proto files change
    println!("cargo:rerun-if-changed=../../protos/billing/v1/billing.proto" );
    Ok(())
}
