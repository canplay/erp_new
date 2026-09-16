import os
import re

repo = r"D:\Workspace\erp_new"
backend = os.path.join(repo, "Backend")

fixed = []
for root, dirs, files in os.walk(backend):
    for filename in files:
        if filename.endswith(".rs"):
            filepath = os.path.join(root, filename)
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original = content
            
            # Fix 1: CRLF -> LF
            content = content.replace('\r\n', '\n')
            
            # Fix 2: String literal followed by , ) ] } ; " -> add space
            # Match a string literal followed by punctuation
            content = re.sub(r'("(?:[^"\\]|\\.)*")\s*([,\)\]\};"])', r'\1 \2', content)
            
            # Fix 3: Missing semicolons before closing brace
            # Pattern: line ending with } or ) or , followed by line starting with }
            # This is harder to fix automatically, so skip for now
            
            if content != original:
                with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
                    f.write(content)
                fixed.append(filename)
                print(f"Fixed: {filename}")

print(f"\nTotal files fixed: {len(fixed)}")
