import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Show the problematic section in detail
print("=== Full context around errors ===")
for i in range(286, 312):
    if i < len(lines):
        marker = ">>>" if i in [288, 289, 290, 305] else "   "
        print(f"{marker} {i+1:3d}: {repr(lines[i])}")

# Check for the pattern: match ... { \n {
# The extra { on the line after match is the problem
print("\n=== Checking for erroneous { after match ===")
for i in range(len(lines) - 1):
    stripped = lines[i].strip()
    if stripped.endswith("match") or stripped.endswith("await {") or stripped.endswith(") {"):
        if i + 1 < len(lines):
            next_line = lines[i + 1].strip()
            if next_line == '{':
                print(f"Line {i+1}: {lines[i].strip()}")
                print(f"Line {i+2}: {lines[i+1].strip()}  <-- REMOVE THIS")
