import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Replace lines 283-297 (0-indexed: 282-296)
# Find the exact lines to replace
start_idx = None
end_idx = None
for i, line in enumerate(lines):
    if 'Ok(user_id) => (' in line and start_idx is None:
        start_idx = i
    if start_idx is not None and '}).into_response(),' in line:
        end_idx = i
        break

print(f"Found block from line {start_idx+1} to {end_idx+1}")

# Replace the block
new_block = '''        Ok(user_id) => {
            let response = serde_json::json!({
                "id": user_id,
                "username": req.username
            });
            (StatusCode::CREATED, json_success(response)).into_response()
        }
'''

# Replace lines start_idx to end_idx (inclusive)
new_lines = lines[:start_idx] + [new_block] + lines[end_idx+1:]

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done")
