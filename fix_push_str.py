import os

repo = r"D:\Workspace\erp_new"
filepath = os.path.join(repo, "Backend/services/user-service/src/http_handlers.rs")

with open(filepath, 'rb') as f:
    content = f.read()

# Find the problematic pattern: \n inside a string followed by )
# Replace with: actual newline escape using format!
# But simpler: just use push('\n') instead of \n in string

# Convert CRLF to LF first
content = content.replace(b'\r\n', b'\n')

# Find and fix the specific pattern
# Original: csv_content.push_str("...\n");
# Fixed: two separate calls
old = b'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at\\n");'
new = b'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at");\n            csv_content.push(\'\\n\');'

if old in content:
    content = content.replace(old, new)
    print("Fixed push_str line")
else:
    print("Pattern not found")

with open(filepath, 'wb') as f:
    f.write(content)

print("Done")
