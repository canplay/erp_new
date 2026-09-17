filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    # Remove lines with just .unwrap_or(0);
    if line.strip() == '.unwrap_or(0);':
        continue
    new_lines.append(line)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done - removed standalone .unwrap_or(0) lines")
