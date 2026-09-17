import re
import subprocess

schema_file = r'D:\Workspace\erp_new\Backend\sql\schema.sql'
with open(schema_file, 'r') as f:
    content = f.read()

# Find all CREATE TABLE statements
tables = {}
pattern = r'CREATE TABLE IF NOT EXISTS "public"\."(\w+)" \((.*?)\);'
for match in re.finditer(pattern, content, re.DOTALL):
    table_name = match.group(1)
    table_body = match.group(2)
    tables[table_name] = f'CREATE TABLE IF NOT EXISTS "public"."{table_name}" ({table_body});'

# Create missing tables
missing = ['tenants', 'pay', 'audit_logs', 'api_usage_logs', 'tenant_users']
for table in missing:
    if table in tables:
        sql = tables[table]
        result = subprocess.run(
            ['podman', 'exec', '-i', 'sesms-postgres', 'psql', '-U', 'postgres', '-d', 'erp_new'],
            input=sql, capture_output=True, text=True
        )
        if result.returncode == 0 or 'CREATE TABLE' in result.stdout or 'already exists' in result.stderr:
            print(f"OK {table}")
        else:
            print(f"FAIL {table}: {result.stderr[:200]}")
    else:
        print(f"NOT FOUND {table}: schema not found")
