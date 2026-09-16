import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    # Fix 1: Line 209 - missing closing paren for tuple
    if i == 208:  # Line 209 (0-indexed: 208)
        line = line.replace('            )', '            ))')
    
    # Fix 2: Line 688 - format! missing closing
    if i == 687:  # Line 688
        line = line.replace('&format!("导入完成: 成功 {),', '&format!("导入完成: 成功 {}"),')
    
    # Fix 3: Line 790 - format! missing closing
    if i == 789:  # Line 790
        line = line.replace('&format!("成功更新 {),', '&format!("成功更新 {}"),')
    
    # Fix 4: Line 704 - \n at end of string (line continuation)
    if i == 703:  # Line 704
        line = line.replace('created_at\\n");', 'created_at");')
        new_lines.append(line)
        # Add the newline as a separate push
        new_lines.append("            csv_content.push('\\n');\n")
        continue
    
    new_lines.append(line)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done")
