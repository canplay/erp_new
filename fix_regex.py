import re

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix 1: json_success(serde_json::json!({...})) -> json_success!({...})
# Pattern: json_success(serde_json::json!( <content> ))
# Replace:  json_success!(<content>)
def fix_json_success(text):
    # Match json_success(serde_json::json!(<content>))
    # where <content> is everything until the matching close paren
    result = []
    i = 0
    while i < len(text):
        # Look for the pattern
        match = re.search(r'json_success\(serde_json::json!\(', text[i:])
        if not match:
            result.append(text[i:])
            break
        
        # Append everything before the match
        result.append(text[i:i+match.start()])
        
        # Find the matching close paren
        start = i + match.end()
        depth = 1
        j = start
        while j < len(text) and depth > 0:
            if text[j] == '(':
                depth += 1
            elif text[j] == ')':
                depth -= 1
            j += 1
        
        # Extract content between outer parens
        inner = text[start:j-1]  # Exclude the final )
        
        # Append the fixed version
        result.append(f'json_success!({inner})')
        
        i = j
    
    return ''.join(result)

content = fix_json_success(content)

# Fix 2: json_success_msg({...}, &format!(...)) -> json_success_msg!({...}, ...)
# This is trickier because of nested braces
def fix_json_success_msg(text):
    result = []
    i = 0
    while i < len(text):
        match = re.search(r'json_success_msg\(\s*\{', text[i:])
        if not match:
            result.append(text[i:])
            break
        
        result.append(text[i:i+match.start()])
        
        # Find the matching close paren for json_success_msg(
        start = i + match.start()
        paren_start = start + text[start:].index('(')
        
        # Find matching close paren
        depth = 1
        j = paren_start + 1
        while j < len(text) and depth > 0:
            if text[j] == '(':
                depth += 1
            elif text[j] == ')':
                depth -= 1
            j += 1
        
        # Extract the full content inside json_success_msg(...)
        full_content = text[paren_start+1:j-1]
        
        # Replace with json_success_msg!(...)
        result.append(f'json_success_msg!({full_content})')
        
        i = j
    
    return ''.join(result)

content = fix_json_success_msg(content)

# Fix 3: Missing closing paren on tuple returns
# Pattern: Ok(x) => (\n    StatusCode::XXX,\n    json_success!(...)
# Need to add ) before .into_response()
# Actually, the issue is that json_success!() needs another ) to close the tuple

# Fix: json_success!(...) followed by )\n .into_response()
# The pattern is: json_success!( <content> )\n            .into_response()
# But the ) closes json_success!, not the tuple
# We need: json_success!( <content> ))\n            .into_response()

# Actually the issue is different. Looking at the code:
# (
#     StatusCode::OK,
#     json_success!(...)
# )
# The outer ( needs a matching ), but json_success!(...) consumes one )

# Wait, let me look at the actual structure:
# Ok(user_id) => (
#     StatusCode::CREATED,
#     json_success(serde_json::json!({...}),
# )
#     .into_response(),

# The issue is: json_success(serde_json::json!({...}) has unbalanced parens
# json_success( - 1 open
# serde_json::json!( - 1 open  
# { - 1 open
# } - 1 close
# ) - closes json_success(
# ) - this should close the tuple

# But wait, that's: json_success( serde_json::json!( {...} ) )
# That's: json_success( ... ) with serde_json::json!( {...} ) as arg
# Plus the tuple's closing )

# Actually the structure is:
# (
#     StatusCode::CREATED,
#     json_success(serde_json::json!({
#         ...
#     })),
# )
#     .into_response(),

# After fix 1, this becomes:
# (
#     StatusCode::CREATED,
#     json_success!({
#         ...
#     }),
# )
#     .into_response(),

# This should be valid Rust...

# Let me just write the fixed content
with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done")
