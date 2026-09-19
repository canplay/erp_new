//! 数据库访问层
//!
//! 提供车辆、仓储、订单、系统配置、用户等实体的 CRUD 操作。

pub mod car;
pub mod common;
pub mod options;
pub mod order;
pub mod storage;
pub mod user;

pub use car::CarRepository;
pub use common::GenericRepository;
pub use common::safe_table_name;
pub use options::OptionsRepository;
pub use order::{OrderRepository, OrderQueryParams};
pub use storage::StorageRepository;
pub use user::UserRepository;
