import sys

filepath = 'FileManagerPage.vue'
with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

first_script_end = content.find('</script>')
style_start = content.find('<style scoped>')
tag_len = len('</script>')

if first_script_end == -1 or style_start == -1:
    print('ERROR: markers not found')
    sys.exit(1)

removed = style_start - first_script_end - tag_len
new_content = content[:first_script_end + tag_len] + '\n\n' + content[style_start:]

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(new_content)

print(f'OK: removed {removed} chars of old code')
print(f'New file: {len(new_content)} chars, {new_content.count(chr(10))} lines')
