import os, re

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# The issue: tuple syntax with json_success(serde_json::json!(...)) is too complex for the parser
# Fix: replace json_success(serde_json::json!({...})) with json_success!({...})
# and json_success_msg({...}, msg) with json_success_msg!(...)

new_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    
    # Pattern 1: json_success(serde_json::json!({...})) -> json_success!({...})
    # Look for the multi-line pattern
    if 'json_success(serde_json::json!(' in line:
        # Collect lines until we find the matching ))
        j = i
        open_count = 0
        full_text = ""
        while j < len(lines):
            full_text += lines[j]
            open_count += lines[j].count('(') + lines[j].count('{') - lines[j].count(')') - lines[j].count('}')
            if open_count <= 0 and j > i:
                break
            j += 1
        
        # Now we have the full expression from line i to line j
        # Pattern: json_success(serde_json::json!( <content> )) \n .into_response()
        # Change to: json_success!(<content>)
        content = full_text
        
        # Extract the inner content of json!()
        # json_success(serde_json::json!( <inner> )) -> json_success!(<inner>)
        match = re.search(r'json_success\(serde_json::json!(\(.*?\))\)', content, re.DOTALL)
        if match:
            inner = match.group(1)
            # Remove the outer () from json!()
            if inner.startswith('(') and inner.endswith(')'):
                inner = inner[1:-1]
            replacement = f'json_success!({inner})'
            full_text = full_text[:match.start()] + replacement + full_text[match.end():]
            
            # Add .into_response() on next line
            new_lines.append(full_text)
            new_lines.append("            .into_response(),\n")
            i = j + 1
            # Skip the original .into_response() line
            if i < len(lines) and '.into_response()' in lines[i]:
                i += 1
            continue
    
    # Pattern 2: json_success_msg({...}, format!(...)) - similar fix
    if 'json_success_msg(' in line and i + 1 < len(lines) and '{' in lines[i+1]:
        # Collect the full expression
        j = i
        full_text = ""
        open_count = 0
        while j < len(lines):
            full_text += lines[j]
            for ch in lines[j]:
                if ch in '({[':
                    open_count += 1
                elif ch in ')}]':
                    open_count -= 1
            if open_count <= 0 and j > i:
                break
            j += 1
        
        # Pattern: json_success_msg( { <content> }, <msg> ) \n .into_response()
        # Change to: json_success_msg!( { <content> }, <msg> )
        content = full_text
        match = re.search(r'json_success_msg\(\s*(\{.*?\})\s*,\s*(.*?)\)', content, re.DOTALL)
        if match:
            inner = match.group(1)
            msg = match.group(2)
            replacement = f'json_success_msg!({inner}, {msg})'
            full_text = content[:match.start()] + replacement + content[match.end():]
            
            new_lines.append(full_text)
            i = j + 1
            if i < len(lines) and '.into_response()' in lines[i]:
                i += 1
            continue
    
    new_lines.append(line)
    i += 1

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done")
