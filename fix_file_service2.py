filepath = r'D:\Workspace\erp_new\Backend\services\file-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Replace .bind() with nothing
content = content.replace('.bind()', '')

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Fixed")
