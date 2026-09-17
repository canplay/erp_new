import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Show the exact problematic section
print("=== Lines 280-305 ===")
for i in range(279, 305):
    if i < len(lines):
        print(f"{i+1:3d}: {repr(lines[i])}")

# Check if the old pattern still exists
old_pattern = 'json_success(serde_json::json!({\n                    "id": user_id,'
for i, line in enumerate(lines):
    if '"id": user_id,' in line:
        print(f"\nFound '\"id\": user_id,' at line {i+1}")
        print(f"  Current line: {repr(line)}")
        if i > 0:
            print(f"  Previous line: {repr(lines[i-1])}")
        if i < len(lines) - 1:
            print(f"  Next line: {repr(lines[i+1])}")
