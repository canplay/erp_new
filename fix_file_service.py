import re

filepath = r'D:\Workspace\erp_new\Backend\services\file-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Remove trailing .bind() at the end of lines (before .fetch_ or .execute)
content = re.sub(r'\.bind\(\)\s*\n(\s*\.(fetch_|execute))', r'\n\1', content)

# Also handle the case where .bind() is at the very end of a line
content = re.sub(r'\.bind\(\)', '', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Fixed file-service repository.rs - removed empty .bind() calls")
