import re

filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Remove .unwrap_or(0) after .await? (since i64 doesn't have unwrap_or)
content = re.sub(r'\.await\?\.unwrap_or\(0\)', '.await?', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
