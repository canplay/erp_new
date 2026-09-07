#!/bin/bash
# =============================================================================
# get-harbor-latest-tag.sh — 查询 Harbor 上最新镜像 tag
# =============================================================================
# 用法: source "$SCRIPT_DIR/lib/get-harbor-latest-tag.sh"
#       get_harbor_latest_tag <image_name> <harbor_url> <project>
# 返回: 最新 tag 或空字符串（如果镜像不存在）
# =============================================================================

get_harbor_latest_tag() {
    local image_name="$1"
    local harbor_url="$2"
    local project="$3"

    # 使用 Harbor API 获取标签列表
    local tags
    tags="$(curl -sk "https://$harbor_url/api/v2.0/projects/$project/repositories/$image_name/artifacts?&page=1&page_size=1" \
        -H "Authorization: Basic $(echo -n "admin:$(harbor_get_password "$harbor_url")" | base64)" \
        2>/dev/null | python3 -c '
import json, sys
try:
    data = json.load(sys.stdin)
    if data and "children" in data:
        # 找到非-digest的tag
        for child in data["children"]:
            if child.get("manifestMediaType") and child.get("tags"):
                print(child["tags"][0])
                break
except:
    pass
' 2>/dev/null)"

    echo "$tags"
}
