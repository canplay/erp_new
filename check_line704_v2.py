import os

filepath = r'D:\Workspace\erp_new\Backend\services\user-service\src\http_handlers.rs'

with open(filepath, 'rb') as f:
    content = f.read()

lines = content.split(b'\n')
line704 = lines[703]

print(f'Line 704 length: {len(line704)}')
print(f'Line 704 hex: {line704.hex()}')

cr_byte = b'\r'
print(f'Has CR: {cr_byte in line704}')
print(f'Last 10 bytes: {line704[-10:].hex()}')
