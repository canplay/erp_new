//! 端到端集成测试
//! 运行: set DATABASE_URL=postgres://myai:myai_secret@localhost:5432/myai && cargo test -p social-ops-service --test end_to_end_test -- --nocapture
// test_bilibili_crawl 已忽略：需要 BrowserPool 模块（未实现）

use sqlx::PgPool;
use uuid::Uuid;
use social_ops_service::services::account_service::AccountService;
use social_ops_service::services::content_service::ContentService;
use social_ops_service::services::llm_service::LlmService;
use social_ops_service::models::account::CreateAccountRequest;

async fn init_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:${DATABASE_PASSWORD}@localhost:5432/myai".to_string());
    PgPool::connect(&url).await.expect("数据库连接失败")
}

#[tokio::test]
async fn test_account_crud() {
    println!("=== 测试: 社交账号 CRUD ===");
    let pool = init_pool().await;
    let svc = AccountService::new(pool.clone());

    // 创建
    let req = CreateAccountRequest {
        user_id: None,
        platform: "bilibili".to_string(),
        account_name: "测试账号".to_string(),
        credentials: None,
        config: None,
    };
    let created = svc.create(&req).await.expect("创建失败");
    println!("✅ 创建账号: {} ({})", created.account_name, created.id);

    // 查询
    let list = svc.list(None).await.expect("查询失败");
    assert!(!list.is_empty(), "账号列表不应为空");
    println!("✅ 账号列表: {} 条", list.len());

    // 更新
    let update_req = social_ops_service::models::account::UpdateAccountRequest {
        account_name: Some("测试账号(已更新)".to_string()),
        config: None,
        is_active: Some(false),
    };
    svc.update(created.id, &update_req).await.expect("更新失败");
    let updated = svc.get(created.id).await.expect("查询失败").unwrap();
    assert_eq!(updated.account_name, "测试账号(已更新)");
    println!("✅ 更新账号: {} -> {}", created.account_name, updated.account_name);

    // 删除
    svc.delete(created.id).await.expect("删除失败");
    let deleted = svc.get(created.id).await.expect("查询失败");
    assert!(deleted.is_none());
    println!("✅ 删除账号成功");

    println!("🎉 账号 CRUD 测试通过!\n");
}

#[tokio::test]
async fn test_content_crud() {
    println!("=== 测试: 内容库 CRUD ===");
    let pool = init_pool().await;
    let svc = ContentService::new(pool.clone());

    // 创建
    let created = svc.create("测试标题", "测试正文内容，这是一段测试文本。", "article", None)
        .await.expect("创建失败");
    println!("✅ 创建内容: {} ({})", created["title"], created["id"]);

    // 查询列表
    let (items, total) = svc.list(None, 1, 10).await.expect("查询失败");
    assert!(total > 0);
    println!("✅ 内容列表: {} 条 (共 {})", items.len(), total);

    // 查询详情
    let id: Uuid = created["id"].as_str().unwrap().parse().unwrap();
    let detail = svc.get(id).await.expect("查询失败").unwrap();
    println!("✅ 内容详情: {}", detail["title"]);

    println!("🎉 内容库 CRUD 测试通过!\n");
}

#[ignore]
#[tokio::test]
async fn test_bilibili_crawl() {
    // 已禁用：需要 BrowserPool 模块（未实现）
    println!("⚠️ 测试已禁用，需要 BrowserPool 模块");
}

#[tokio::test]
async fn test_llm_provider_crud() {
    println!("=== 测试: LLM 提供商 CRUD ===");
    let pool = init_pool().await;
    let svc = LlmService::new(pool.clone());

    // 创建
    let provider = svc.add_provider("测试LLM", "https://api.openai.com/v1", "sk-test123", "gpt-4o")
        .await.expect("创建失败");
    println!("✅ 创建 LLM 提供商: {} ({})", provider["provider_name"], provider["id"]);

    // 列表
    let list = svc.list_providers().await.expect("查询失败");
    assert!(!list.is_empty());
    println!("✅ LLM 提供商列表: {} 条", list.len());

    // 删除
    let id: Uuid = provider["id"].as_str().unwrap().parse().unwrap();
    svc.delete_provider(id).await.expect("删除失败");
    println!("✅ 删除 LLM 提供商成功");

    println!("🎉 LLM 提供商测试通过!\n");
}
