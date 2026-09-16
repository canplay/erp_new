import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# The core issue: json_success!({ was replaced with serde_json::json!()
# But serde_json::json!() doesn't include the outer { } that json_success!({ had
# So we need to wrap the json!() calls with { }

# Fix: json_success(serde_json::json!(...)) -> json_success({ ... })
# Actually, looking at the code:
# serde_json::json!({...}) produces a Value::Object
# But json_success!({...}) was a macro that included the wrapping braces

# The fix: wrap serde_json::json!() result in a block to provide the missing context
# json_success(serde_json::json!({...})) should be json_success(serde_json::json!({...}))
# The issue is missing opening { before serde_json::json!

# Pattern: json_success(serde_json::json!({
# This is correct - the { is part of serde_json::json! macro

# But looking at the error, the issue is different.
# The functions like list_users have:
# match ... {
#     Ok(result) => {
#         ...
#         (  <- This opens a tuple, not a block
#             StatusCode::OK,
#             json_success(...)
#         )  <- closes tuple
#         .into_response()
#     }  <- This SHOULD close the Ok arm
# But wait, there's a missing } somewhere

# Let me check if there's a pattern where json_success!({ was replaced incorrectly
# The old code was: json_success!({ "key": value })
# The new code is: json_success(serde_json::json!({ "key": value }))
# This should be fine...

# Actually the issue might be different. Let me check for functions that return
# (StatusCode, Json) tuples without proper block braces

# Search for patterns where match arms have ( instead of {
import re

lines = content.split('\n')
fixed_lines = []
in_function = False
brace_depth = 0

for i, line in enumerate(lines):
    stripped = line.strip()
    
    # Track brace depth
    brace_depth += line.count('{') - line.count('}')
    
    # Check for match arms that use ( instead of {
    if stripped.startswith('Ok(') or stripped.startswith('Err(') or stripped == '(_, _) => (':
        # Check if next non-empty line starts with StatusCode
        for j in range(i+1, min(i+3, len(lines))):
            if 'StatusCode' in lines[j]:
                # This is a tuple return, not a block - add opening {
                line = line.replace('=> (', '=> {')
                # We'll need to add closing } before the end
                break
    
    fixed_lines.append(line)

# Actually, the real issue is simpler. Let me look at what the compiler says:
# Line 311: } mismatched - this is the end of a function
# Line 535: } mismatched - end of function
# etc.

# The issue is that some match arms have ( instead of { for their body
# When the arm body is a tuple, Rust expects either:
# 1. A block: Ok(x) => { (StatusCode, Json) }
# 2. Direct tuple: Ok(x) => (StatusCode, Json)

# If we have Ok(x) => { followed by a tuple without closing }, that's an error

# Let me just add missing closing braces
for i, line in enumerate(lines):
    stripped = line.strip()
    # Check for tuple returns in match arms
    if '=> (' in stripped:
        # Find the matching closing )
        for j in range(i+1, min(i+10, len(lines))):
            if lines[j].strip() == ')':
                # Check if next line is .into_response()
                if j+1 < len(lines) and '.into_response()' in lines[j+1]:
                    # Need to add closing } after .into_response()
                    pass
                break

# Simpler approach: just fix the specific error lines
# The pattern is: Ok(result) => { ... ( ... ) ... .into_response() }
# Missing: } before the end of the Ok arm

# Fix: for each function, ensure all match arms are properly closed
# Let's check for the specific pattern

# Pattern 1: list_users
content = content.replace(
    '            )' + '\n' + '                .into_response()' + '\n' + '        }',
    '            ))' + '\n' + '                .into_response()' + '\n' + '        }'
)

# Actually, let me just check the exact structure and fix each case
print(f"Total lines: {len(lines)}")

# Check lines around the errors
for line_num in [305, 306, 307, 308, 309, 310, 311]:
    if line_num <= len(lines):
        print(f"Line {line_num}: {lines[line_num-1]}")
