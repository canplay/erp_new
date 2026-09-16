import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Show full context around problematic areas
print("=== Lines 195-215 ===")
for i in range(194, 215):
    print(f"{i+1:3d}: {lines[i]}", end='')

print("\n=== Lines 520-540 ===")
for i in range(519, 540):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}", end='')

print("\n=== Lines 670-695 ===")
for i in range(669, 695):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}", end='')

print("\n=== Lines 785-802 ===")
for i in range(784, 802):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}", end='')
