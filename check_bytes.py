import os

filepath = r"D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs"

# Read as bytes
with open(filepath, 'rb') as f:
    content = f.read()

# Find line 704
lines = content.split(b'\n')
print(f"Total lines: {len(lines)}")
print(f"Line 704 raw: {lines[703]}")

# Check bytes around position 80-95
print(f"Bytes 70-95: {lines[703][70:95].hex()}")

# The issue: the \n (0x5c 0x6e) might have something between them
# Let's look at the actual bytes
line = lines[703]
for i in range(len(line)-1):
    if line[i:i+2] == b'\\n':  # backslash + n
        print(f"Found backslash+n at position {i}: context = {line[max(0,i-5):i+10]}")
    if line[i:i+1] == b'\\':
        print(f"Found backslash at position {i}: context = {line[max(0,i-3):i+10]}")
