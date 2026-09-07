//! 微博抓取真实集成测试（调试版）
//! 运行: cargo test -p social-ops-service --test weibo_crawl_test -- --nocapture
//!
//! 已禁用：需要 drisson crate 和 BrowserPool 模块（均未实现）
//! 要启用，需要：
//! 1. 在 social-ops-service/Cargo.toml 中添加 drisson 依赖
//! 2. 在 social-ops-service 中实现 BrowserPool 模块

// TODO: 依赖就绪后恢复此测试
