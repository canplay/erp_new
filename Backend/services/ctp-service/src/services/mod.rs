//! 业务逻辑层 — CTP 平板锁协议

mod ctp_device;
mod status_parser;

pub use ctp_device::*;
pub use status_parser::parse_status;
