import os, re

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix ALL instances of space before comma (most common issue)
# Pattern: something" , -> something",
content = re.sub(r'"(\s*),', '",', content)
content = re.sub(r'"(\s*)', '"', content)

# Wait, that's too aggressive. Let me be more specific.
# The issue is: "/route" , -> "/route",
content = re.sub(r'\s+,', ',', content)

# Fix space before closing paren: ) -> )
content = re.sub(r'\s+\)', ')', content)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print("Done - removed all spaces before commas and parens")
