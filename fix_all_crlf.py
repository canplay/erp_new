import os
import subprocess

repo = r"D:\Workspace\erp_new"

# Disable autocrlf globally
subprocess.run(["git", "config", "--local", "core.autocrlf", "false"], cwd=repo)

# Remove all .rs files from index and reset
result = subprocess.run(["git", "ls-files", "*.rs"], cwd=repo, capture_output=True, text=True)
files = [f for f in result.stdout.strip().split('\n') if f]

# Remove from index
for f in files:
    subprocess.run(["git", "rm", "--cached", f], cwd=repo, capture_output=True)

# Force checkout with LF
subprocess.run(["git", "checkout", "HEAD", "--"] + files, cwd=repo, capture_output=True)

# Now fix all .rs files - convert CRLF to LF
fixed = []
for root, dirs, filenames in os.walk(os.path.join(repo, "Backend")):
    for filename in filenames:
        if filename.endswith(".rs"):
            filepath = os.path.join(root, filename)
            try:
                with open(filepath, 'rb') as f:
                    content = f.read()
                if b'\r\n' in content:
                    content = content.replace(b'\r\n', b'\n')
                    with open(filepath, 'wb') as f:
                        f.write(content)
                    fixed.append(filename)
            except:
                pass

print(f"Fixed CRLF in {len(fixed)} files")
for f in fixed[:10]:
    print(f"  - {f}")
