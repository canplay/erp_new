//! 用户信息服务
//!
//! 通过内部服务调用获取用户信息

use serde::{Deserialize, Serialize};

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub real_name: Option<String>,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,
}

/// 用户服务客户端
///
/// 用于从 user-service 获取用户信息
#[derive(Clone)]
pub struct UserServiceClient {
    /// 用户服务 HTTP 地址（内部调用）
    user_service_url: String,
}

impl UserServiceClient {
    #[must_use]
    pub const fn new(user_service_url: String) -> Self {
        Self { user_service_url }
    }

    /// 根据用户ID获取用户信息
    pub async fn get_user_by_id(&self, user_id: i64) -> Option<UserInfo> {
        let url = format!("{}/api/internal/users/{}" , self.user_service_url, user_id);

        match reqwest::get(&url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            let data = json.get("data" )?;
                            Some(UserInfo {
                                id: data.get("id" )?.as_i64()?,
                                username: data.get("username" )?.as_str()?.to_string(),
                                real_name: data
                                    .get("real_name" )
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                avatar: data
                                    .get("avatar" )
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                email: data.get("email" ).and_then(|v| v.as_str()).map(String::from),
                                phone: data.get("phone" ).and_then(|v| v.as_str()).map(String::from),
                                role: data.get("role" ).and_then(|v| v.as_str()).map(String::from),
                            })
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// 根据用户ID列表批量获取用户信息
    pub async fn get_users_by_ids(&self, user_ids: &[i64]) -> Vec<UserInfo> {
        if user_ids.is_empty() {
            return vec![];
        }

        let url = format!("{}/api/internal/users/batch" , self.user_service_url);
        let body = serde_json::json!({ "ids": user_ids });

        match reqwest::Client::new().post(&url).json(&body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            let data = json.get("data" ).and_then(|v| v.as_array());
                            data.map(|arr| {
                                arr.iter()
                                    .filter_map(|item| {
                                        Some(UserInfo {
                                            id: item.get("id" )?.as_i64()?,
                                            username: item.get("username" )?.as_str()?.to_string(),
                                            real_name: item
                                                .get("real_name" )
                                                .and_then(|v| v.as_str())
                                                .map(String::from),
                                            avatar: item
                                                .get("avatar" )
                                                .and_then(|v| v.as_str())
                                                .map(String::from),
                                            email: item
                                                .get("email" )
                                                .and_then(|v| v.as_str())
                                                .map(String::from),
                                            phone: item
                                                .get("phone" )
                                                .and_then(|v| v.as_str())
                                                .map(String::from),
                                            role: item
                                                .get("role" )
                                                .and_then(|v| v.as_str())
                                                .map(String::from),
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default()
                        }
                        Err(_) => vec![],
                    }
                } else {
                    vec![]
                }
            }
            Err(_) => vec![],
        }
    }
}

/// 带用户信息关联的 CMS 状态
#[derive(Clone)]
pub struct CmsWithUserInfo {
    pub repository: crate::CmsRepository,
    pub user_client: Option<UserServiceClient>,
}

impl CmsWithUserInfo {
    pub fn new(repository: crate::CmsRepository, user_service_url: Option<String>) -> Self {
        Self {
            repository,
            user_client: user_service_url.map(UserServiceClient::new),
        }
    }

    /// 获取用户信息
    pub async fn get_user_info(&self, user_id: i64) -> Option<UserInfo> {
        if let Some(ref client) = self.user_client {
            client.get_user_by_id(user_id).await
        } else {
            None
        }
    }

    /// 批量获取用户信息
    pub async fn get_users_info(&self, user_ids: &[i64]) -> Vec<UserInfo> {
        if let Some(ref client) = self.user_client {
            client.get_users_by_ids(user_ids).await
        } else {
            vec![]
        }
    }
}
