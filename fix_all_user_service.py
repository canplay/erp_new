import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Fix each line
new_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    
    # Fix 1: Remove space before ) on push_str line
    if 'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at" );' in line:
        line = line.replace('" );', '");')
        new_lines.append(line)
        new_lines.append("            csv_content.push('\\n');\n")
        i += 1
        continue
    
    # Fix 2: Fix spaces before commas in headers
    line = line.replace('"Content-Type" , "text/csv; charset=utf-8" )', '"Content-Type", "text/csv; charset=utf-8")')
    line = line.replace('"Content-Disposition" ,', '"Content-Disposition",')
    line = line.replace('"attachment; filename=users_export.csv" ,', '"attachment; filename=users_export.csv",')
    
    # Fix 3: Fix format! missing closing paren
    line = line.replace('&format!("导入完成: 成功 {),', '&format!("导入完成: 成功 {}"),')
    line = line.replace('&format!("成功更新 {),', '&format!("成功更新 {}"),')
    
    # Fix 4: Fix ImportUsersRequest struct
    if 'pub struct ImportUsersRequest pub content:' in line:
        line = line.replace('pub struct ImportUsersRequest pub content: String,', 'pub struct ImportUsersRequest {')
        new_lines.append(line)
        new_lines.append('    pub content: String,\n')
        i += 1
        continue
    
    # Fix 5: Fix tuple closing - line 209 needs extra )
    # Check if this is the line with the closing ) after json_success
    if i == 208:  # Line 209 (0-indexed: 208)
        # This line should be: "            )" 
        # But we need: "            ))" to close both json_success and tuple
        line = line.replace('            )', '            ))', 1)
    
    new_lines.append(line)
    i += 1

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done")
