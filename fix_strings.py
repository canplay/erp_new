import re

filepath = r"D:\Workspace\erp_new\Backend\services\user-service\src\role_handlers.rs"

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix: "string" followed by , or ) or " -> "string" followed by space+separator
# Pattern: match a string literal (double-quoted) followed by a comma, close paren, or another string
# Insert a space between them
content = re.sub(r'("(?:[^"\\]|\\.)*")([,\)"\[])', r'\1 \2', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
