import os

filepath = r"D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs"

# Read as bytes
with open(filepath, 'rb') as f:
    content = f.read()

# Check CRLF
crlf = content.count(b'\r\n')
lf = content.count(b'\n') - crlf
print(f"Before: CRLF={crlf}, LF={lf}")

# Replace CRLF with LF
content = content.replace(b'\r\n', b'\n')

# Find and fix the problematic line (line 704)
lines = content.split(b'\n')
for i, line in enumerate(lines):
    if b'csv_content.push_str' in line and b'created_at' in line:
        # Replace with properly escaped \\n
        lines[i] = b'            csv_content.push_str("id,username,nickname,email,phone,role,status,created_at\\n");'
        print(f"Fixed line {i+1}")
        break

content = b'\n'.join(lines)

# Write as bytes with LF only
with open(filepath, 'wb') as f:
    f.write(content)

# Verify
crlf = content.count(b'\r\n')
lf = content.count(b'\n') - crlf
print(f"After: CRLF={crlf}, LF={lf}")
print("Done")
