import os

filepath = r"D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs"

# Read raw bytes
with open(filepath, 'rb') as f:
    data = f.read()

# Convert to bytearray for mutation
data = bytearray(data)

# Find the problematic pattern: "created_at\n");
# We need to find this in the context of push_str
# The pattern is: created_at\n");
# Which bytes: 63 72 65 61 74 65 64 5f 61 74 5c 6e 22 29 3b
# We want: created_at\\n");
# Which bytes: 63 72 65 61 74 65 64 5f 61 74 5c 5c 6e 22 29 3b

# Find the pattern
pattern = b'created_at\\n");'
replacement = b'created_at\\\\n");'

if pattern in data:
    data = data.replace(pattern, replacement)
    print(f"Found and fixed pattern")
else:
    print(f"Pattern not found")
    # Try to find it
    idx = data.find(b'created_at')
    if idx >= 0:
        print(f"Found 'created_at' at position {idx}")
        print(f"Context: {data[idx:idx+30]}")

# Write back
with open(filepath, 'wb') as f:
    f.write(data)

print("Done")
