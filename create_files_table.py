import re
import subprocess

with open(r'D:\Workspace\erp_new\Backend\sql\schema.sql', 'r') as f:
    content = f.read()

pattern = r'CREATE TABLE IF NOT EXISTS "public"\."(\w+)" \((.*?)\);'
tables = {}
for match in re.finditer(pattern, content, re.DOTALL):
    tables[match.group(1)] = f'CREATE TABLE IF NOT EXISTS "public"."{match.group(1)}" ({match.group(2)});'

# Create remaining missing tables
for table in ['files']:
    if table in tables:
        result = subprocess.run(
            ['podman', 'exec', '-i', 'sesms-postgres', 'psql', '-U', 'postgres', '-d', 'erp_new'],
            input=tables[table], capture_output=True, text=True
        )
        print(f"{'OK' if result.returncode == 0 else 'FAIL'} {table}: {result.stderr[:100] if result.returncode != 0 else ''}")
    else:
        print(f"NOT FOUND {table}")
