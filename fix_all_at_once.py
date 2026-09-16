import os, subprocess, re

repo = r'D:\Workspace\erp_new'
backend = os.path.join(repo, 'Backend')

# Disable autocrlf
subprocess.run(['git', 'config', '--local', 'core.autocrlf', 'false'], cwd=repo)

# Get all .rs files from git
result = subprocess.run(['git', 'ls-files', 'Backend/**/*.rs'], cwd=repo, capture_output=True, text=True)
files = [f for f in result.stdout.strip().split('\n') if f]

for filepath in files:
    fullpath = os.path.join(repo, filepath)
    
    # Get raw content from git
    result = subprocess.run(['git', 'show', f'HEAD:{filepath}'], cwd=repo, capture_output=True)
    content = result.stdout.decode('utf-8')
    
    # Fix 1: CRLF -> LF
    content = content.replace('\r\n', '\n')
    
    # Fix 2: \n at end of string -> split into two statements  
    if 'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at\\n");' in content:
        content = content.replace(
            'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at\\n");',
            'csv_content.push_str("id,username,nickname,email,phone,role,status,created_at");\n            csv_content.push(\'\\n\');'
        )
    
    # Fix 3: string literal followed by , ) ] } " -> add space (Rust 2021+)
    content = re.sub(r'("(?:[^"\\]|\\.)*")([,\)\]\}"])', r'\1 \2', content)
    
    # Fix 4: format! missing closing paren
    content = content.replace('&format!("成功更新 {),', '&format!("成功更新 {}"),')
    
    # Fix 5: ImportUsersRequest missing opening brace  
    content = content.replace(
        'pub struct ImportUsersRequest pub content: String,\n}',
        'pub struct ImportUsersRequest {\n    pub content: String,\n}'
    )
    
    # Write with LF endings
    with open(fullpath, 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)

print(f'Processed {len(files)} files')
