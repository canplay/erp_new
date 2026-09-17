import re

filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# The issue is that query_scalar::<_, i64> returns i64, not Option<i64>
# When the result is empty, it fails. We need to use fetch_optional instead.
# For now, let's just remove .unwrap_or(0) from query_scalar results

# Pattern: .fetch_one(&self.pool).await?.unwrap_or(0) -> .fetch_one(&self.pool).await?
content = re.sub(r'\.await\?\.unwrap_or\(0\)', '.await?', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
