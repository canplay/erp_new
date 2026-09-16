filepath = r"D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs"

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find and fix line 704
for i, line in enumerate(lines):
    if 'csv_content.push_str' in line and 'created_at' in line and 'nickname' in line:
        # Replace with two lines to avoid any escaping issues
        lines[i] = '            csv_content.push_str("id,username,nickname,email,phone,role,status,created_at");\n'
        lines.insert(i+1, '            csv_content.push(\'\\n\');\n')
        print(f"Fixed line {i+1}")
        break

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(lines)

print("Done")
