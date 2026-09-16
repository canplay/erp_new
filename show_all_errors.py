import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Show the problematic section
print("=== Lines 289-311 ===")
for i in range(288, 311):
    print(f"{i+1:3d}: {lines[i]}", end='')

print("\n=== Lines 525-535 ===")
for i in range(524, 535):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}", end='')

print("\n=== Lines 677-691 ===")
for i in range(676, 691):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}", end='')

print("\n=== Lines 785-800 ===")
for i in range(784, 800):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}", end='')
