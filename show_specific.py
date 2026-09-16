import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

print(f"Total lines: {len(lines)}")

# Show specific problematic lines
for i in [200, 208, 289, 525, 529, 533, 676, 687, 703, 724, 726, 727, 789, 796, 913]:
    if i < len(lines):
        print(f"Line {i+1}: {repr(lines[i])}")
