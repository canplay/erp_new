import os
import re

repo = r"D:\Workspace\erp_new"
backend = os.path.join(repo, "Backend")

fixed = []
for root, dirs, files in os.walk(backend):
    for filename in files:
        if filename.endswith(".rs"):
            filepath = os.path.join(root, filename)
            with open(filepath, 'rb') as f:
                content = f.read()
            
            # Convert to string for processing
            text = content.decode('utf-8')
            
            # Fix 1: CRLF -> LF
            text = text.replace('\r\n', '\n')
            
            # Fix 2: String literal followed by , ) ] } ; " -> add space
            text = re.sub(r'("(?:[^"\\]|\\.)*")([,\)\]\};"])', r'\1 \2', text)
            
            # Fix 3: line continuation inside string literals (backslash + actual newline)
            # Pattern: a string that ends with backslash followed by newline
            # Replace with backslash + escaped n
            def fix_line_continuation(match):
                return match.group(0)[:-1] + '\\n' + '\n'
            
            # This is tricky - we need to find lines where backslash + newline appears inside a string
            # A simpler approach: find all backslash followed by newline and replace with \\n
            # But only if the backslash is odd (meaning it's not an escaped backslash)
            lines = text.split('\n')
            new_lines = []
            for i, line in enumerate(lines):
                # Check if this line ends with a single backslash
                if line.endswith('\\') and not line.endswith('\\\\'):
                    # Check if we're inside a string by counting unescaped quotes
                    quote_count = 0
                    j = 0
                    while j < len(line):
                        if line[j] == '\\':
                            j += 2
                        elif line[j] == '"':
                            quote_count += 1
                            j += 1
                        else:
                            j += 1
                    
                    if quote_count % 2 == 1:
                        # Inside string, replace \ + newline with \\ + n + newline
                        line = line[:-1] + '\\n'
                
                new_lines.append(line)
            
            text = '\n'.join(new_lines)
            
            # Write back
            with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
                f.write(text)
            fixed.append(filename)
            print(f"Fixed: {filename}")

print(f"\nTotal files fixed: {len(fixed)}")
