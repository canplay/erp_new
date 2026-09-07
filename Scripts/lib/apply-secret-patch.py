#!/usr/bin/env python3
"""apply-secret-patch.py — 将密钥值写入JSON patch文件

用法:
  apply-secret-patch.py <key=value>... <patch_file>

每个 key=value 中的 value 如果是 URL 格式(含://或: @)则自动做 URL-encode。
"""
import json
import sys
import urllib.parse
import re

def url_encode_if_needed(value):
    """如果值包含 URL 特殊字符则做 URL-encode"""
    if re.search(r'[/:@#?&=+]', value):
        return urllib.parse.quote(value, safe='')
    return value

def main():
    if len(sys.argv) < 3:
        print("Usage: apply-secret-patch.py <key=value>... <patch_file>", file=sys.stderr)
        sys.exit(1)

    kv_pairs = {}
    patch_file = sys.argv[-1]

    for arg in sys.argv[1:-1]:
        if '=' not in arg:
            print(f"ERROR: 无效的参数格式: {arg}", file=sys.stderr)
            sys.exit(1)
        key, value = arg.split('=', 1)
        # URL-encoding for values that look like URLs
        if '://' in value or (':' in value and '@' in value):
            value = url_encode_if_needed(value)
        kv_pairs[key] = value

    patch = {"stringData": kv_pairs}

    with open(patch_file, "w", encoding="utf-8") as f:
        json.dump(patch, f, ensure_ascii=False, indent=2)

if __name__ == "__main__":
    main()
