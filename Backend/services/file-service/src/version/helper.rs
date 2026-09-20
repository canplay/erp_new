//! 版本管理内部辅助函数

/// 生成版本 ID
pub(crate) fn generate_version_id(file_id: &str, version_number: u32) -> String {
    format!("{file_id}_v{version_number:03}")
}
