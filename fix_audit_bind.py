filepath = r'D:\Workspace\erp_new\Backend\services\audit-service\src\repository.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Remove all .bind() occurrences (empty bind calls)
import re
content = re.sub(r'\.bind\(\)', '', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done - removed all empty .bind() calls")
