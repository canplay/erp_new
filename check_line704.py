filepath = r"D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs"

with open(filepath, 'rb') as f:
    content = f.read()

# Check CRLF
crlf = content.count(b'\r\n')
lf = content.count(b'\n') - crlf
print(f"CRLF: {crlf}, LF: {lf}")

# Show line 704 bytes
lines = content.split(b'\n')
line = lines[703]
print(f"Line 704 length: {len(line)} bytes")
print(f"Line 704 hex: {line.hex()}")
print(f"Line 704 repr: {line}")

# Check for \r in line 704
if b'\r' in line:
    print("FOUND \\r in line 704!")
else:
    print("No \\r in line 704")

# Check last 10 bytes
print(f"Last 10 bytes: {line[-10:].hex()}")
print(f"Last 10 chars: {line[-10:]}")
