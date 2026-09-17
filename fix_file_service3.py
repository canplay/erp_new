filepath = r'D:\Workspace\erp_new\Backend\services\file-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    # Remove .bind() at end of line (before optional whitespace)
    if line.rstrip().endswith('.bind()'):
        line = line.rstrip()[:-7] + '\n'  # Remove .bind() (7 chars)
    new_lines.append(line)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done")
