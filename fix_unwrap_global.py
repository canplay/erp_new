import os
import re

backend = r'D:\Workspace\erp_new\Backend'

fixed = 0
for root, dirs, files in os.walk(backend):
    for filename in files:
        if filename.endswith('.rs'):
            filepath = os.path.join(root, filename)
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original = content
            
            # Fix: .await?.unwrap_or(0) -> .await? (for non-Option types like i64)
            content = re.sub(r'\.await\?\.unwrap_or\(0\)', '.await?', content)
            content = re.sub(r'\.await\?\.unwrap_or_default\(\)', '.await?', content)
            
            if content != original:
                with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
                    f.write(content)
                fixed += 1

print(f"Fixed {fixed} files")
