"""
Clean remaining any types - replace with proper types or eslint-disable
"""
import re
import os

# Files with any types to clean
files_with_any = [
    "Frontend/apps/admin/src/components/ImportDialog/ImportDialog.vue",
    "Frontend/apps/admin/src/components/ImportDialog/ImportPreview.vue",
    "Frontend/apps/admin/src/components/ImportDialog/Main.vue",
    "Frontend/apps/admin/src/components/permission/FieldMaskConfig/FieldMaskConfig.vue",
    "Frontend/apps/admin/src/components/permission/FieldMaskTable.vue",
    "Frontend/apps/admin/src/components/permission/RowPermissionConfig/Main.vue",
    "Frontend/apps/admin/src/components/permission/RowPermissionConfig/useRowPermission.ts",
    "Frontend/apps/admin/src/components/TreeSelector/Main.vue",
    "Frontend/apps/admin/src/components/VirtualScrollTable/Main.vue",
    "Frontend/apps/admin/src/components/VirtualScrollTable/TableHeader.vue",
    "Frontend/apps/admin/src/components/VirtualScrollTable.vue",
    "Frontend/apps/admin/src/pages/lpr/LiveMonitorPage.vue",
    "Frontend/apps/admin/src/pages/report/ReportViewPage.vue",
    "Frontend/apps/admin/src/pages/system/AnnouncementList.vue",
    "Frontend/apps/admin/src/pages/user/UserListPage.vue",
    "Frontend/apps/admin/src/stores/ctp.ts",
    "Frontend/apps/admin/src/utils/alova.ts",
]

base_dir = "D:/Workspace/erp_new/"

total_cleaned = 0

for rel_path in files_with_any:
    file_path = os.path.join(base_dir, rel_path)
    if not os.path.exists(file_path):
        continue
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # Pattern 1: (something as unknown as any) as any -> remove outer as any
    content = re.sub(r'\(unknown as any\) as any', 'unknown as unknown', content)
    
    # Pattern 2: (something as unknown as any) as any as Type -> (something as unknown) as Type
    content = re.sub(r'\(unknown as any\) as (\w+)', 'unknown as \\1', content)
    
    # Pattern 3: (something as any) -> something (when simple expression)
    content = re.sub(r'\((\w+) as any\)', '\\1', content)
    
    # Pattern 4: something as any -> keep but add eslint-disable
    # For now, replace with unknown where safe
    content = re.sub(r'\s+as any\b', ' as unknown', content)
    
    # Pattern 5: undefined as any -> undefined
    content = re.sub(r'undefined as any', 'undefined', content)
    
    # Pattern 6: const x = ... as any -> const x = ...
    content = re.sub(r'= (.+) as any\b', '= \\1', content)
    
    # Pattern 7: : any -> : Record<string, unknown> for config-like objects
    content = re.sub(r': any\b', ': unknown', content)
    
    if content != original_content:
        # Count replacements
        diff = len(original_content.split('\n')) - len(content.split('\n'))
        total_cleaned += abs(diff) if diff > 0 else 1
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Cleaned: {os.path.basename(rel_path)}")

print(f"\nTotal files cleaned: {total_cleaned}")
