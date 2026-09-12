"""
Process API files to add error handling - Handles multiline generics
"""
import re
import os

api_dir = "D:/Workspace/erp_new/Frontend/apps/admin/src/api"

api_files = [
    "apiGovernance.ts", "api-key.ts", "clean.ts", "cms.ts", "ctp.ts",
    "department.ts", "device.ts", "dictionary.ts", "exportTask.ts",
    "feedback.ts", "file.ts", "ip-whitelist.ts", "log.ts", "lpr.ts",
    "message.ts", "notification.ts", "operation-log.ts", "pay.ts",
    "permission.ts", "report.ts", "role.ts", "scheduled-task.ts",
    "sensitive-audit.ts", "system.ts", "tenant.ts", "tow.ts",
    "user.ts", "workflow.ts", "xlt.ts"
]

context_map = {
    "apiGovernance": "API治理",
    "api-key": "API密钥管理",
    "clean": "清运服务",
    "cms": "CMS内容管理",
    "ctp": "CTP设备管理",
    "department": "部门管理",
    "device": "设备管理",
    "dictionary": "字典管理",
    "exportTask": "导出任务",
    "feedback": "意见反馈",
    "file": "文件管理",
    "ip-whitelist": "IP白名单",
    "log": "日志管理",
    "lpr": "车牌识别",
    "message": "站内信",
    "notification": "通知公告",
    "operation-log": "操作日志",
    "pay": "支付服务",
    "permission": "权限管理",
    "report": "报表管理",
    "role": "角色管理",
    "scheduled-task": "定时任务",
    "sensitive-audit": "敏感操作审计",
    "system": "系统配置",
    "tenant": "租户管理",
    "tow": "拖车服务",
    "user": "用户管理",
    "workflow": "工作流",
    "xlt": "XLT停车管理",
}


def find_matching_paren(text, start_pos):
    """Find the matching closing paren for an opening paren at start_pos"""
    depth = 0
    pos = start_pos
    while pos < len(text):
        if text[pos] == '(':
            depth += 1
        elif text[pos] == ')':
            depth -= 1
            if depth == 0:
                return pos
        pos += 1
    return -1


def find_matching_brace(text, start_pos):
    """Find the matching closing brace for an opening brace at start_pos"""
    depth = 0
    pos = start_pos
    while pos < len(text):
        if text[pos] == '{':
            depth += 1
        elif text[pos] == '}':
            depth -= 1
            if depth == 0:
                return pos
        pos += 1
    return -1


def extract_httpclient_call(body_lines, return_line_idx):
    """
    Extract httpClient call from body lines starting at return_line_idx.
    Properly handles multiline generic types and args.
    Returns (method, args_text, end_line_idx) or (None, None, None).
    """
    return_line = body_lines[return_line_idx]
    
    # Find httpClient.method(
    http_match = re.search(r'httpClient\.(get|post|put|delete)', return_line)
    if not http_match:
        return None, None, None
    
    method = http_match.group(1)
    
    # Find the opening paren after method name, skipping generic type params
    # Generic types can span multiple lines: httpClient.get<{\n  list: X[];\n}>(
    paren_pos = -1
    angle_depth = 0
    
    # Search across lines for the opening paren
    for idx in range(return_line_idx, len(body_lines)):
        line = body_lines[idx]
        start_pos = http_match.end() if idx == return_line_idx else 0
        
        for i in range(start_pos, len(line)):
            if line[i] == '<':
                angle_depth += 1
            elif line[i] == '>':
                angle_depth -= 1
            elif line[i] == '(' and angle_depth == 0:
                paren_pos = i
                paren_line_idx = idx
                break
        
        if paren_pos != -1:
            break
    
    if paren_pos == -1:
        return None, None, None
    
    # Find matching close paren - may span multiple lines
    full_args = ""
    paren_depth = 1
    angle_depth = 0
    end_line_idx = paren_line_idx
    
    for idx in range(paren_line_idx, len(body_lines)):
        line = body_lines[idx]
        start_pos = paren_pos + 1 if idx == paren_line_idx else 0
        
        for i in range(start_pos, len(line)):
            if line[i] == '(' and angle_depth <= 0:
                paren_depth += 1
            elif line[i] == ')' and angle_depth <= 0:
                paren_depth -= 1
                if paren_depth == 0:
                    # Found matching close
                    full_args += line[start_pos:i]
                    return method, full_args, idx
            elif line[i] == '<':
                angle_depth += 1
            elif line[i] == '>':
                angle_depth -= 1
        
        if idx == paren_line_idx:
            full_args += line[paren_pos+1:]
        else:
            full_args += '\n' + line
        
        end_line_idx = idx
    
    return None, None, None


def process_file(file_path, context_name):
    """Add try/catch with handleApiError to API functions"""

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Skip if already has error handling
    if 'handleApiError' in content:
        return 0, "already handled"

    # Add import at the top
    import_line = "import { handleApiError } from '@/utils/apiErrorHandler';\n"
    if "apiErrorHandler" not in content:
        lines = content.split('\n')
        last_import_idx = -1
        for idx, line in enumerate(lines):
            if line.strip().startswith('import '):
                last_import_idx = idx

        if last_import_idx >= 0:
            lines.insert(last_import_idx + 1, import_line.rstrip())
            content = '\n'.join(lines)

    # Find all export function declarations (including async)
    pattern = r'export (?:async )?function (\w+)\('

    matches = list(re.finditer(pattern, content))
    changes = 0

    # Process from end to start to preserve positions
    for match in reversed(matches):
        func_name = match.group(1)

        # Find the opening paren
        paren_search_start = match.end() - 1
        paren_close = find_matching_paren(content, paren_search_start)
        if paren_close == -1:
            continue

        # Find the opening brace after the closing paren
        brace_search_start = paren_close + 1
        brace_pos = content.find('{', brace_search_start)
        if brace_pos == -1:
            continue

        brace_end = find_matching_brace(content, brace_pos)
        if brace_end == -1:
            continue

        func_body = content[brace_pos:brace_end+1]

        # Check if body contains httpClient call
        if 'httpClient.' not in func_body:
            continue

        # Check if it's a simple return statement
        body_lines = func_body.split('\n')

        # Find the return line
        return_line_idx = -1
        for idx, line in enumerate(body_lines):
            stripped = line.strip()
            if stripped.startswith('return httpClient.'):
                return_line_idx = idx
                break

        if return_line_idx == -1:
            continue

        return_line = body_lines[return_line_idx]
        indent = len(return_line) - len(return_line.lstrip())
        indent_str = ' ' * indent

        # Extract the httpClient call
        method, full_args, end_line_idx = extract_httpclient_call(body_lines, return_line_idx)
        
        if method is None:
            continue

        # Build new function body
        new_body_lines = [
            '{',
            f'{indent_str}try {{',
            f'{indent_str}  return await httpClient.{method}({full_args});',
            f'{indent_str}}} catch (error) {{',
            f"{indent_str}  handleApiError(error, '{context_name}');",
            f'{indent_str}  throw error;',
            f'{indent_str}}}',
            '}'
        ]
        new_body = '\n'.join(new_body_lines)

        # Replace in content
        content = content[:brace_pos] + new_body + content[brace_end+1:]
        changes += 1

    if changes > 0:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)

    return changes, "processed"


total_changes = 0
for fname in api_files:
    fpath = os.path.join(api_dir, fname)
    if os.path.exists(fpath):
        context = context_map.get(fname.replace('.ts', ''), fname.replace('.ts', ''))
        changes, status = process_file(fpath, context)
        total_changes += changes
        if changes > 0:
            print(f"{fname}: {changes} functions updated")

print(f"\nTotal: {total_changes} API methods got error handling")
