fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取当前 crate 的根目录
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    // protos 目录在 Backend/protos/，相对于 crates/grpc-proto/ 需要 ../../protos/
    let protos_dir = std::path::Path::new(crate_dir).join("../../protos");
    let protos_dir_str = protos_dir.to_string_lossy();

    let protos = &[
        format!("{protos_dir_str}/auth.proto"),
        format!("{protos_dir_str}/user.proto"),
        format!("{protos_dir_str}/cms.proto"),
        format!("{protos_dir_str}/message.proto"),
        format!("{protos_dir_str}/file.proto"),
        format!("{protos_dir_str}/feedback.proto"),
        format!("{protos_dir_str}/tenant.proto"),
        format!("{protos_dir_str}/workflow.proto"),
        format!("{protos_dir_str}/audit.proto"),
        format!("{protos_dir_str}/api_key.proto"),
        format!("{protos_dir_str}/clean.proto"),
        format!("{protos_dir_str}/browser.proto"),
        format!("{protos_dir_str}/ctp.proto"),
        format!("{protos_dir_str}/lpr.proto"),
        format!("{protos_dir_str}/tow.proto"),
        format!("{protos_dir_str}/socialops.proto"),
        format!("{protos_dir_str}/hik.proto"),
        format!("{protos_dir_str}/xlt.proto"),
        format!("{protos_dir_str}/ebike.proto"),
        format!("{protos_dir_str}/pay.proto"),
    ];
    let includes = &[protos_dir_str.to_string()];

    // tonic-prost-build 0.14: 配置并编译 proto 文件，生成 server 和 client
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(protos, includes)?;
    Ok(())
}
