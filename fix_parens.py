import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix: json_success(serde_json::json!(...)) missing closing )
# Pattern: json_success(serde_json::json!( {...} )
# Should be: json_success(serde_json::json!( {...} ))

# Fix 1: get_import_template
content = content.replace(
    'json_success(serde_json::json!( {' + '\n' + '                "template": template,',
    'json_success(serde_json::json!({' + '\n' + '                "template": template,'
)

# Fix 2: import_users - json_success_msg missing closing )
# Pattern: json_success_msg({...}, &format!(...),
content = content.replace(
    'json_success_msg({' + '\n' + '                "total": total,',
    'json_success_msg({' + '\n' + '                "total": total,'
)

# Fix 3: batch_update_user_role - json_success_msg missing closing )
content = content.replace(
    '&format!("成功更新 {}"),' + '\n' + '        ).into_response(),',
    '&format!("成功更新 {}"),' + '\n' + '    )' + '\n' + '        .into_response(),'
)

# Fix 4: batch_delete_users - json_success_msg missing closing )
content = content.replace(
    '&format!("成功删除 {}"),' + '\n' + '        ).into_response(),',
    '&format!("成功删除 {}"),' + '\n' + '    )' + '\n' + '        .into_response(),'
)

# Fix 5: create_user - json_success missing closing )
content = content.replace(
    'json_success(serde_json::json!( {' + '\n' + '                    "id": user_id,',
    'json_success(serde_json::json!({' + '\n' + '                    "id": user_id,'
)

# Fix 6: list_users - json_success missing closing )
content = content.replace(
    'json_success(serde_json::json!( {' + '\n' + '                        "list": users,',
    'json_success(serde_json::json!({' + '\n' + '                        "list": users,'
)

# Fix 7: export_users - json_success missing closing )
content = content.replace(
    'json_success(serde_json::json!( {' + '\n' + '                        "list": users,',
    'json_success(serde_json::json!({' + '\n' + '                        "list": users,'
)

# Actually, the issue is more fundamental. The json_success! macro was replaced
# with serde_json::json!() but the wrapping is wrong.
# Let me fix by adding the missing closing paren for json_success(

# Pattern: json_success(serde_json::json!( {...} )
# The ) at the end closes json_success, but we need another ) to close the tuple

# Simpler fix: find all json_success(serde_json::json!( and ensure they have matching parens
lines = content.split('\n')
fixed_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    
    # Check if this line has json_success(serde_json::json!(
    if 'json_success(serde_json::json!(' in line or 'json_success_msg(' in line:
        # Count parens in this line
        open_parens = line.count('(')
        close_parens = line.count(')')
        
        # If more open than close, we need to find the closing
        if open_parens > close_parens:
            # Look ahead for the closing
            j = i + 1
            while j < len(lines) and j < i + 15:
                open_parens += lines[j].count('(')
                close_parens += lines[j].count(')')
                
                if close_parens >= open_parens:
                    # Found the closing, but we need to add ) for json_success
                    # The closing ) is for json_success, but we need one more for the tuple
                    lines[j] = lines[j].replace(')', '))', 1)
                    break
                j += 1
    
    fixed_lines.append(line)
    i += 1

content = '\n'.join(fixed_lines)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
