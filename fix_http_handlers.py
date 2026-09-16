import os, subprocess, re

repo = r'D:\Workspace\erp_new'
filepath = os.path.join(repo, 'Backend/services/user-service/src/http_handlers.rs')

# Extract raw bytes from git
result = subprocess.run(['git', 'show', 'HEAD:Backend/services/user-service/src/http_handlers.rs'], cwd=repo, capture_output=True)
content = result.stdout.decode('utf-8')

# Fix 1: CRLF -> LF
content = content.replace('\r\n', '\n')

# Fix 2: \n at end of string -> split into two statements  
content = content.replace('csv_content.push_str("id,username,nickname,email,phone,role,status,created_at\\n");',
                          'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at");\n            csv_content.push(\'\\n\');')

# Fix 3: string literal followed by , ) ] } " -> add space
content = re.sub(r'("(?:[^"\\]|\\.)*")([,\)\]\}"])', r'\1 \2', content)

# Fix 4: format! missing closing paren
content = content.replace('&format!("成功更新 {),', '&format!("成功更新 {}"),')

# Fix 5: ImportUsersRequest missing opening brace
content = content.replace('pub struct ImportUsersRequest pub content: String,\n}',
                          'pub struct ImportUsersRequest {\n    pub content: String,\n}')

# Write
with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.write(content)

print('Done')
