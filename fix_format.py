import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'r', encoding='utf-8') as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    # Fix: format! missing closing brace and paren
    line = line.replace('&format!("导入完成: 成功 {),', '&format!("导入完成: 成功 {}"),')
    line = line.replace('&format!("成功更新 {),', '&format!("成功更新 {}"),')
    line = line.replace('&format!("成功删除 {),', '&format!("成功删除 {}"),')
    line = line.replace('&format!("成功创建 {),', '&format!("成功创建 {}"),')
    line = line.replace('&format!("成功 {),', '&format!("成功 {}"),')
    
    new_lines.append(line)

with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
    f.writelines(new_lines)

print("Done")
