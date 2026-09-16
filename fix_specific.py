import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix all issues:

# 1. Remove space before ) on line 704
content = content.replace('csv_content.push_str("id,username,nickname,email,phone,role,status,created_at" );',
                          'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at");')

# 2. Fix spaces before commas in headers (lines 725-728)
content = content.replace('"Content-Type" , "text/csv; charset=utf-8" )',
                          '"Content-Type", "text/csv; charset=utf-8")')
content = content.replace('"Content-Disposition" ,',
                          '"Content-Disposition",')
content = content.replace('"attachment; filename=users_export.csv" ,',
                          '"attachment; filename=users_export.csv",')

# 3. Fix missing closing parens - line 201
# The issue is: Ok(result) => {\n            let users: Vec<UserResponse> = result.users.into_iter().map(std::convert::Into::into).collect();\n            (
# Missing closing } for the Ok arm

# Let me look at specific line ranges
lines = content.split('\n')
print(f"Total lines: {len(lines)}")

# Show lines around errors
for line_num in [199, 200, 201, 210, 211, 289, 290, 310, 311, 523, 524, 525, 526, 534, 535, 677, 678, 799, 800]:
    if line_num <= len(lines):
        print(f"Line {line_num}: {lines[line_num-1]}")

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("\nDone")
