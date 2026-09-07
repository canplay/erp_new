fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&["../../protos/cms.proto"], &["../../protos"])?;
    Ok(())
}
