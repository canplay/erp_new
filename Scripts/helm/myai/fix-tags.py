#!/usr/bin/env python3
"""Fix tag fields in values-services.yaml for each service."""

import re

with open('/d/Workspace/MyAI/Scripts/helm/myai/values-services.yaml', 'r') as f:
    lines = f.readlines()

# Map of service name -> tag
service_tags = {
    'apiGateway': '20260824141000',
    'authService': '202608142030',
    'userService': '202608142030',
    'cmsService': '202608142030',
    'messageService': '202608142030',
    'fileService': '202608142030',
    'tenantService': '202608142030',
    'workflowService': '202608142030',
    'feedbackService': '202608142030',
    'auditService': '202608142030',
    'apiKeyService': '202608142030',
    'hikService': '202608142030',
    'payService': '202608142030',
    'cleanService': '202608142030',
    'towService': '202608142030',
    'ctpService': '202608142030',
    'xltService': '202608142030',
    'ebikeService': '202608142030',
    'admin': '20260824124711',
}

# Find all service definitions and their tags
# Each service definition is 2-space indented, each tag is 4-space indented
services = {}  # service_name -> (start_line, tag_line_or_None)
current_service = None
for i, line in enumerate(lines):
    stripped = line.strip()
    # Check if this is a service definition (2-space indent, ends with colon)
    if re.match(r'^  [a-zA-Z]+:', line) and not line.startswith('    '):
        current_service = stripped.split(':')[0].strip()
        if current_service in service_tags:
            services[current_service] = (i, None)
    
    # Check if this is a tag line (4-space indent, contains 'tag:')
    if current_service and current_service in service_tags:
        if 'tag:' in stripped and line.startswith('    ') and not line.startswith('      '):
            if services[current_service][1] is None:
                services[current_service] = (services[current_service][0], i)

# Now rebuild the file
# For each service, ensure exactly ONE tag entry with the correct value
output = []
i = 0
while i < len(lines):
    line = lines[i]
    stripped = line.strip()
    
    # Check if this is a service definition we care about
    svc_match = re.match(r'^  ([a-zA-Z]+):', line)
    if svc_match and not line.startswith('    '):
        svc_name = svc_match.group(1)
        if svc_name in service_tags:
            # Add the service definition line
            output.append(line)
            i += 1
            
            # Now process all lines for this service until the next service definition
            # We need to ensure exactly ONE tag entry
            has_tag = False
            tag_line_idx = None
            temp_lines = []
            
            while i < len(lines):
                l = lines[i]
                s = l.strip()
                
                # Check if this is the next service definition
                if re.match(r'^  [a-zA-Z]+:', l) and not l.startswith('    '):
                    break
                
                # Check if this is a tag line
                if 'tag:' in s and l.startswith('    ') and not l.startswith('      '):
                    if not has_tag:
                        # First tag - use the correct value
                        tag = service_tags[svc_name]
                        output.append(f'    tag: "{tag}"')
                        has_tag = True
                        tag_line_idx = len(output) - 1
                    # Skip this line (duplicate)
                    i += 1
                    continue
                
                temp_lines.append(l)
                i += 1
            
            # If no tag was found, insert one after 'enabled:' line
            if not has_tag:
                # Find 'enabled:' in temp_lines
                enabled_idx = None
                for j, tl in enumerate(temp_lines):
                    if 'enabled:' in tl.strip():
                        enabled_idx = j
                        break
                
                if enabled_idx is not None:
                    # Insert tag after enabled:
                    tag = service_tags[svc_name]
                    temp_lines.insert(enabled_idx + 1, f'    tag: "{tag}"')
            
            # Add all the other lines
            output.extend(temp_lines)
            continue
        else:
            # Not a service we care about, just add the line
            output.append(line)
            i += 1
            continue
    
    output.append(line)
    i += 1

# Write back
with open('/d/Workspace/MyAI/Scripts/helm/myai/values-services.yaml', 'w') as f:
    f.writelines(output)

print("Fixed values-services.yaml")
print(f"Services with tags: {list(service_tags.keys())}")

# Verify each service has exactly one tag
for svc_name in service_tags:
    found = False
    for line in output:
        if f'    tag: "{service_tags[svc_name]}"' in line.strip():
            found = True
            break
    status = 'OK' if found else 'MISSING'
    print(f"  {svc_name}: {status}")
