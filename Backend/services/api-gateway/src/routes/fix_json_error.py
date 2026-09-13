import os
import re

routes_dir = "D:/Workspace/erp_new/Backend/services/api-gateway/src/routes"

def process_file(path):
    with open(path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original = content
    replaced_count = 0
    
    # All patterns to find and replace
    patterns = [
        ("SERVICE_UNAVAILABLE", "认证服务不可用: {e}", True),
        ("SERVICE_UNAVAILABLE", "用户服务不可用: {e}", True),
        ("SERVICE_UNAVAILABLE", "CMS服务不可用: {e}", True),
        ("INTERNAL_SERVER_ERROR", "获取分类失败: {e}", True),
        ("INTERNAL_SERVER_ERROR", "获取分类树失败: {e}", True),
        ("INTERNAL_SERVER_ERROR", "获取文章列表失败: {e}", True),
        ("INTERNAL_SERVER_ERROR", "获取用户信息失败: {e}", True),
        ("INTERNAL_SERVER_ERROR", "获取文章失败: {e}", True),
        ("INTERNAL_SERVER_ERROR", "生成刷新令牌失败: {}", True),
        ("INTERNAL_SERVER_ERROR", "未知错误: {e}", True),
        ("INTERNAL_SERVER_ERROR", "密码修改失败: {e}", True),
        ("INTERNAL_SERVER_ERROR", "更新资料失败: {e}", True),
        ("INTERNAL_SERVER_ERROR", "导出待实现", False),
        ("BAD_REQUEST", "创建分类失败: {e}", True),
        ("BAD_REQUEST", "更新分类失败: {e}", True),
        ("BAD_REQUEST", "删除分类失败: {e}", True),
        ("BAD_REQUEST", "创建文章失败: {e}", True),
        ("BAD_REQUEST", "更新文章失败: {e}", True),
        ("BAD_REQUEST", "删除文章失败: {e}", True),
        ("BAD_REQUEST", "登录失败: {e}", True),
        ("BAD_REQUEST", "注册失败: {e}", True),
        ("BAD_REQUEST", "密码修改失败: {e}", True),
        ("BAD_REQUEST", "新密码必须不少于 8 位且同时包含字母和数字", False),
        ("BAD_REQUEST", "参数错误: {e}", True),
        ("NOT_FOUND", "规则不存在", False),
        ("NOT_FOUND", "审计记录不存在", False),
        ("NOT_FOUND", "记录不存在", False),
        ("NOT_FOUND", "任务不存在", False),
        ("NOT_FOUND", "资源不存在: {e}", True),
        ("NOT_FOUND", "获取文章失败: {e}", True),
        ("UNAUTHORIZED", "原密码验证失败", False),
        ("UNAUTHORIZED", "Token刷新失败: {e}", True),
    ]
    
    for status_code, msg, has_e in patterns:
        if has_e:
            old = "(StatusCode::" + status_code + ", json_error(&format!(\"" + msg + "\")))"
            new = "(StatusCode::" + status_code + ", json_error_response_fmt(StatusCode::" + status_code + ", \"" + msg + "\", &format!(\"" + msg + "\")))"
        else:
            old = "(StatusCode::" + status_code + ", json_error(&format!(\"" + msg + "\")))"
            new = "(StatusCode::" + status_code + ", json_error_response(StatusCode::" + status_code + ", \"" + msg + "\"))"
        
        count = content.count(old)
        if count > 0:
            content = content.replace(old, new)
            replaced_count += count
            print(f"  Replaced {count} of '{old[:80]}...'")
    
    if content != original:
        with open(path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"  -> {path}: {replaced_count} replacements")
    else:
        print(f"  -> {path}: no changes")

# Process all route files (except helpers and social_ops)
for f in sorted(os.listdir(routes_dir)):
    if f.endswith('.rs') and f not in ('helpers.rs', 'social_ops_routes.rs'):
        path = os.path.join(routes_dir, f)
        with open(path, 'r', encoding='utf-8') as fh:
            content = fh.read()
        # Only process files that have json_error in map_err context
        if 'json_error(' in content:
            print(f"\nProcessing {f}...")
            process_file(path)
