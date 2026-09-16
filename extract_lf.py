import os
import subprocess

repo = r"D:\Workspace\erp_new"

# Step 1: Disable autocrlf
subprocess.run(["git", "config", "--local", "core.autocrlf", "false"], cwd=repo, check=True)

# Step 2: Get list of all .rs files in Backend
result = subprocess.run(
    ["git", "ls-files", "Backend/**/*.rs"],
    cwd=repo, capture_output=True, text=True
)
files = [f for f in result.stdout.strip().split('\n') if f]

# Step 3: For each file, extract from git and write with LF
fixed = []
for filepath in files:
    # Get file content from git
    result = subprocess.run(
        ["git", "show", f"HEAD:{filepath}"],
        cwd=repo, capture_output=True
    )
    content = result.stdout
    
    # Check if content has CRLF
    if b'\r\n' in content:
        fixed.append(os.path.basename(filepath))
    
    # Write with LF only
    with open(os.path.join(repo, filepath), 'wb') as f:
        f.write(content)

print(f"Extracted {len(files)} files from git")
print(f"Files with CRLF: {len(fixed)}")

# Step 4: Verify
with open(os.path.join(repo, "Backend/services/user-service/src/http_handlers.rs"), 'rb') as f:
    content = f.read()
    crlf = content.count(b'\r\n')
    print(f"http_handlers.rs: CRLF count = {crlf}")

print("Done")
