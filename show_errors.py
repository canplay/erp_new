import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Show problematic sections
lines = content.split('\n')

# Show lines around error locations
print("=== Lines 196-214 ===")
for i in range(195, 214):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}")

print("\n=== Lines 286-314 ===")
for i in range(285, 314):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}")

print("\n=== Lines 520-538 ===")
for i in range(519, 538):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}")

print("\n=== Lines 673-685 ===")
for i in range(672, 690):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}")

print("\n=== Lines 795-805 ===")
for i in range(794, 806):
    if i < len(lines):
        print(f"{i+1:3d}: {lines[i]}")
