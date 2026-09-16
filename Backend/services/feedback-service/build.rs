//! 构建脚本
//!
//! 用于编译 proto 文件（如果服务使用 gRPC）
//! 示例：编译 feedback.proto
//!
//! ```ignore
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     tonic_build::configure()
//!         .build_server(true)
//!         .build_client(true)
//!         .compile(
//!             &["../../protos/feedback.proto" ],
//!             &["../../protos" ]
//!         )?;
//!     Ok(())
//! }
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
