import os
import subprocess

repo = r"D:\Workspace\erp_new"
backend = os.path.join(repo, "Backend")

# Ensure autocrlf is disabled
subprocess.run(["git", "config", "--local", "core.autocrlf", "false"], cwd=repo, check=True)

# Remove and re-checkout all .rs files
for root, dirs, files in os.walk(backend):
    for f in files:
        if f.endswith(".rs"):
            filepath = os.path.join(root, f)
            relpath = os.path.relpath(filepath, repo)
            # Remove the file
            os.remove(filepath)
            # Re-checkout
            subprocess.run(["git", "checkout", "HEAD", "--", relpath], cwd=repo, capture_output=True)

# Verify
with open(os.path.join(backend, "services/user-service/src/http_handlers.rs"), 'rb') as f:
    content = f.read()
    if b'\r\n' in content:
        print("WARNING: CRLF still present!")
    else:
        print("All clear - LF only")

print("Done")
