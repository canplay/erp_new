import re

filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Remove .unwrap_or(0) after fetch_one().await? for i64 results
content = re.sub(r'\.fetch_one\(&self\.pool\)\s*\.await\?\s*\.unwrap_or\(0\)', '.fetch_one(&self.pool).await?', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
