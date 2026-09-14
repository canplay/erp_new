//! API Gateway 请求重试模块
//!
//! 提供指数退避、可配置重试次数、重试条件判断

use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::{Duration, Instant};

/// 重试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始退避时间（毫秒）
    pub initial_backoff_ms: u64,
    /// 最大退避时间（毫秒）
    pub max_backoff_ms: u64,
    /// 指数基数
    pub backoff_base: f64,
    /// 是否添加抖动
    pub jitter: bool,
    /// 重试条件
    pub retry_on: Vec<RetryCondition>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 30000,
            backoff_base: 2.0,
            jitter: true,
            retry_on: vec![
                RetryCondition::StatusCode(429),
                RetryCondition::StatusCode(500),
                RetryCondition::StatusCode(502),
                RetryCondition::StatusCode(503),
                RetryCondition::StatusCode(504),
                RetryCondition::Timeout,
                RetryCondition::ConnectionError,
            ],
        }
    }
}

/// 重试条件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryCondition {
    /// 基于状态码重试
    StatusCode(u16),
    /// 状态码范围重试
    StatusCodeRange { start: u16, end: u16 },
    /// 超时重试
    Timeout,
    /// 连接错误重试
    ConnectionError,
    /// 网络错误重试
    NetworkError,
    /// 服务器错误重试 (5xx)
    ServerError,
    /// 始终重试（用于测试）
    Always,
}

impl RetryCondition {
    /// 检查是否满足重试条件
    #[must_use]
    pub const fn matches(&self, error: &RetryableError) -> bool {
        match (self, error) {
            (Self::StatusCode(code), RetryableError::StatusCode(s)) => *code == *s,
            (Self::StatusCodeRange { start, end }, RetryableError::StatusCode(s)) => {
                *s >= *start && *s <= *end
            }
            (Self::Timeout, RetryableError::Timeout) => true,
            (Self::ConnectionError, RetryableError::ConnectionError(_)) => true,
            (Self::NetworkError, RetryableError::NetworkError(_)) => true,
            (Self::ServerError, RetryableError::StatusCode(s)) => *s >= 500 && *s < 600,
            (Self::Always, _) => true,
            _ => false,
        }
    }
}

/// 可重试的错误类型
#[derive(Debug, Clone)]
pub enum RetryableError {
    /// HTTP 状态码错误
    StatusCode(u16),
    /// 超时错误
    Timeout,
    /// 连接错误
    ConnectionError(String),
    /// 网络错误
    NetworkError(String),
    /// 其他错误
    Other(String),
}

impl From<reqwest::Error> for RetryableError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else if err.is_connect() {
            Self::ConnectionError(err.to_string())
        } else if let Some(status) = err.status() {
            Self::StatusCode(status.as_u16())
        } else {
            Self::NetworkError(err.to_string())
        }
    }
}

impl From<Response> for RetryableError {
    fn from(response: Response) -> Self {
        Self::StatusCode(response.status().as_u16())
    }
}

/// 重试结果
#[derive(Debug, Clone)]
pub struct RetryResult<T> {
    /// 最终结果（成功或最后一次失败的错误）
    pub result: Result<T, RetryableError>,
    /// 是否成功
    pub success: bool,
    /// 总尝试次数
    pub attempts: u32,
    /// 总耗时（毫秒）
    pub total_duration_ms: u64,
    /// 各次尝试的耗时（毫秒）
    pub attempt_durations: Vec<u64>,
}

impl<T> RetryResult<T> {
    /// 创建成功结果
    pub fn success(result: T, attempts: u32, durations: Vec<u64>) -> Self {
        Self {
            result: Ok(result),
            success: true,
            attempts,
            total_duration_ms: durations.iter().sum(),
            attempt_durations: durations,
        }
    }

    /// 创建失败结果
    #[must_use]
    pub fn failure(result: RetryableError, attempts: u32, durations: Vec<u64>) -> Self {
        Self {
            result: Err(result),
            success: false,
            attempts,
            total_duration_ms: durations.iter().sum(),
            attempt_durations: durations,
        }
    }
}

/// 重试器
pub struct Retryer {
    config: RetryConfig,
}

impl Default for Retryer {
    fn default() -> Self {
        Self::new(RetryConfig::default())
    }
}

impl Retryer {
    /// 创建新的重试器
    #[must_use]
    pub const fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建重试器
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(RetryConfig::default())
    }

    /// 计算退避时间
    #[must_use]
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base_delay = self.config.initial_backoff_ms as f64;
        let multiplier = self.config.backoff_base.powi(attempt as i32 - 1);
        let delay = base_delay * multiplier;

        // 限制最大退避时间
        let delay = delay.min(self.config.max_backoff_ms as f64);

        // 添加抖动以避免雷群效应
        if self.config.jitter {
            use std::time::SystemTime;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| {
                    tracing::error!("系统时间早于 UNIX epoch，使用默认值");
                    std::time::Duration::from_secs(0)
                })
                .as_nanos();
            let jitter_factor = (((now % 1000) as f64) / 1000.0).mul_add(0.5, 1.0); // 0.5-1.0 的抖动
            Duration::from_millis((delay * jitter_factor) as u64)
        } else {
            Duration::from_millis(delay as u64)
        }
    }

    /// 检查是否应该重试
    /// 允许最多 `max_retries` + 1 次尝试（1次初始 + `max_retries` 次重试）
    #[must_use]
    pub fn should_retry(&self, error: &RetryableError, attempts: u32) -> bool {
        if attempts > self.config.max_retries {
            return false;
        }

        for condition in &self.config.retry_on {
            if condition.matches(error) {
                return true;
            }
        }

        false
    }

    /// 执行带重试的请求
    pub async fn execute<F, Fut, T>(&self, mut request_fn: F) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, RetryableError>>,
    {
        let mut attempts = 0u32;
        let mut attempt_durations = Vec::new();

        loop {
            let attempt_start = Instant::now();
            attempts += 1;

            match request_fn().await {
                Ok(result) => {
                    let duration = attempt_start.elapsed().as_millis() as u64;
                    return RetryResult::success(result, attempts, {
                        let mut durations = attempt_durations;
                        durations.push(duration);
                        durations
                    });
                }
                Err(error) => {
                    let duration = attempt_start.elapsed().as_millis() as u64;
                    attempt_durations.push(duration);

                    tracing::warn!(
                        "请求失败 (尝试 {}/{}): {:?}",
                        attempts,
                        self.config.max_retries,
                        error
                    );

                    if !self.should_retry(&error, attempts) {
                        tracing::error!("达到最大重试次数 {}，放弃重试", self.config.max_retries);
                        return RetryResult::failure(error, attempts, attempt_durations);
                    }

                    // 计算并等待退避时间
                    let backoff = self.calculate_backoff(attempts);
                    tracing::info!(
                        "等待 {}ms 后进行第 {} 次重试...",
                        backoff.as_millis(),
                        attempts + 1
                    );

                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

}

/// 创建带重试的请求配置
#[must_use]
pub fn create_retry_config(max_retries: u32, initial_backoff_ms: u64) -> RetryConfig {
    RetryConfig {
        max_retries,
        initial_backoff_ms,
        max_backoff_ms: 30000,
        backoff_base: 2.0,
        jitter: true,
        retry_on: vec![
            RetryCondition::StatusCode(429),
            RetryCondition::StatusCode(500),
            RetryCondition::StatusCode(502),
            RetryCondition::StatusCode(503),
            RetryCondition::StatusCode(504),
            RetryCondition::Timeout,
            RetryCondition::ConnectionError,
        ],
    }
}

/// 简化版重试函数
pub async fn retry_request<R, F, T>(
    request_fn: R,
    config: RetryConfig,
) -> Result<T, RetryableError>
where
    R: Fn() -> F,
    F: Future<Output = Result<T, RetryableError>>,
{
    let retryer = Retryer::new(config);
    let result = retryer.execute(request_fn).await;
    result.result
}

/// 使用指数退避重试
pub async fn retry_with_backoff<R, F, T>(
    request_fn: R,
    max_retries: u32,
) -> Result<T, RetryableError>
where
    R: Fn() -> F,
    F: Future<Output = Result<T, RetryableError>>,
{
    let config = create_retry_config(max_retries, 100);
    retry_request(request_fn, config).await
}

/// 创建 gRPC 客户端并自动重试
pub async fn create_grpc_client_with_retry<T, F, Fut>(
    create_fn: F,
    retry_config: &RetryConfig,
) -> Result<T, RetryableError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, RetryableError>>,
{
    let retryer = Retryer::new(retry_config.clone());
    let result = retryer.execute(create_fn).await;
    result.result
}
