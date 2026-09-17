import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find and fix all occurrences of the pattern:
# Ok(result) => (
#     StatusCode::XXX,
#     json_success(serde_json::json!({
#             ...
#         }),
# )
#     .into_response(),

new_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    
    # Check for the pattern: Ok(result) => (
    if 'Ok(result) => (' in line:
        # Collect the full block
        j = i + 1
        block = line
        while j < len(lines):
            block += lines[j]
            if '.into_response(),' in lines[j] or '.into_response()\n' in lines[j]:
                break
            j += 1
        
        # Now fix the block
        # Extract the status code and json content
        if 'json_success(serde_json::json!(' in block:
            # Replace json_success(serde_json::json!(...)) with json_success!(...)
            import re
            # Find the json content
            match = re.search(r'json_success\(serde_json::json!\((\{.*?\})\)\)', block, re.DOTALL)
            if match:
                json_content = match.group(1)
                # Replace with intermediate variable
                old = match.group(0)
                new = f'{{ let response = serde_json::json!{json_content}; json_success(response) }}'
                block = block.replace(old, new)
        
        new_lines.append(block)
        i = j + 1
        continue
    
    new_lines.append(line)
    i += 1

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done")
