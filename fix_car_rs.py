import os

# Fix car.rs - all columns in tow_car are likely nullable
filepath = r'D:\Workspace\erp_new\Backend\services\tow-service\src\repository\car.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Replace all pub xxx: String with pub xxx: Option<String>
# But keep id as i64
import re
content = re.sub(r'pub (\w+): String', r'pub \1: Option<String>', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Fixed car.rs - String -> Option<String>")
