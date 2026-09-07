// 员工数据模型
// Staff data model

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 员工记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Staff {pub id: String, pub activiti_sync: Option<String>, pub browser: Option<String>, pub realname: Option<String>, pub status: Option<i32>, pub userkey: Option<String>, pub username: Option<String>, pub departid: Option<String>, pub location: Option<String>}

/// 员工查询参数
#[derive(Debug, Deserialize)]
pub struct StaffQuery {pub id: String}
