import re

filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# The pattern is .await?\n.unwrap_or(0);  (multi-line)
content = re.sub(r'\.await\?\n\s*\.unwrap_or\(0\);', '.await?;', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done - fixed multi-line unwrap_or")
