import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Show the specific problematic lines with their byte content
for i in [289, 290, 291, 292, 293, 294, 295, 296, 297, 298, 299, 300, 301, 302, 303, 304, 305, 306, 307, 308, 309, 310]:
    if i < len(lines):
        line = lines[i]
        # Show hex of critical chars
        print(f"Line {i+1} ({len(line)} chars): {repr(line)}")

# Also show line 305 specifically
print("\n=== Line 305 bytes ===")
line305 = lines[304]
for j, ch in enumerate(line305):
    print(f"  {j}: {repr(ch)} (0x{ord(ch):02x})")
