import os
import re

backend = r'D:\Workspace\erp_new\Backend'

# Fix all .rs files in the project
fixed = 0
for root, dirs, files in os.walk(backend):
    for filename in files:
        if filename.endswith('.rs'):
            filepath = os.path.join(root, filename)
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original = content
            
            # Fix 1: Remove .bind() with no arguments
            content = re.sub(r'\.bind\(\)', '', content)
            
            # Fix 2: Fix sqlx::query_scalar without type annotations
            # Add ::<Postgres, _> to sqlx::query_scalar("...")
            content = re.sub(
                r'sqlx::query_scalar\("(.*?)"\)',
                r'sqlx::query_scalar::<_, _>("\1")',
                content
            )
            content = re.sub(
                r'sqlx::query_scalar\(r#"(.*?)#"\)',
                r'sqlx::query_scalar::<_, _>(r#"\1"#)',
                content
            )
            
            # Fix 3: Remove .unwrap_or(0) after .await? for non-Option types
            content = re.sub(r'\.await\?\.unwrap_or\(0\)', '.await?', content)
            
            # Fix 4: Remove .flatten() calls (not available in this Rust version)
            content = re.sub(r'\.flatten\(\)', '', content)
            
            if content != original:
                with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
                    f.write(content)
                fixed += 1
                print(f"Fixed: {filename}")

print(f"\nTotal files fixed: {fixed}")
