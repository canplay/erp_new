import re

filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix: i64 doesn't have unwrap_or, only Option<i64> does
# Replace .fetch_one().await?.unwrap_or(0) with proper handling
content = re.sub(r'\.fetch_one\(&self\.pool\)\s*\.await\?\s*\.unwrap_or\(0\)', '.fetch_one(&self.pool).await?', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done - removed unwrap_or on i64")
