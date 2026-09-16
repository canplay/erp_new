filepath = r"D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs"

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find and fix line 704 - missing semicolon
for i, line in enumerate(lines):
    if 'csv_content.push_str' in line and 'created_at' in line and 'nickname' in line:
        # Add missing semicolon
        lines[i] = line.rstrip('\n') + ';\n'
        print(f"Fixed line {i+1}: {lines[i].rstrip()}")
        break

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(lines)

print("Done")
