import re

filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix 1: Remove all .bind() occurrences
content = re.sub(r'\.bind\(\)', '', content)

# Fix 2: Fix sqlx::query_scalar without type annotations
# Pattern: sqlx::query_scalar("...") or sqlx::query_scalar(r#"..."#)
# Need to add ::<Postgres, _> type annotation
content = re.sub(
    r'sqlx::query_scalar\((r?".*?")\)',
    lambda m: f'sqlx::query_scalar({m.group(1)})::<Postgres, _>',
    content
)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
