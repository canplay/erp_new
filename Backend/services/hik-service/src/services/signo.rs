// Signo停车场服务
// Signo parking service

use crate::error::{HikError, Result};
use crate::models::SignoConfig;

/// Signo服务
pub struct SignoService {
    config: SignoConfig,
}

impl SignoService {
    /// 创建Signo服务实例
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SignoConfig::new(),
        }
    }

    /// 远程开闸
    pub async fn open_gate(&self, place: &str, march_no: &str) -> Result<String> {
        let tag = self.config.tags.get(place).ok_or(HikError::InvalidParams)?;

        let park_no = &tag.phone;
        let pkey = &tag.key;

        // 构建URL
        let url = format!(
            "http://www.etpcar.com/App/UpOpenCmd.aspx?ParkNo={park_no}&CarNo=&CarType=&MachNo={march_no}&Memo=远程开闸OpenFlg=0"
        )
        .to_lowercase();

        let pkey_lower = pkey.to_lowercase();

        // 计算MD5 token
        let token_input = format!("{url}{pkey_lower}");
        let token = format!("{:x}", md5::compute(token_input.as_bytes()));
        let full_url = format!("{url}&Token={token}");

        // 发送请求
        let client = reqwest::Client::new();
        let resp = client
            .get(&full_url)
            .send()
            .await
            .map_err(HikError::RequestError)?;

        if resp.status().is_success() {
            let text = resp.text().await.map_err(HikError::RequestError)?;
            Ok(text)
        } else {
            Err(HikError::ApiError("调用失败".to_string()))
        }
    }
}

impl Default for SignoService {
    fn default() -> Self {
        Self::new()
    }
}
