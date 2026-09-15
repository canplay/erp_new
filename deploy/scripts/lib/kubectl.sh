#!/bin/bash
# =============================================================================
# Kubectl 操作函数库
# =============================================================================
# 提供 K8s 集群操作、Pod 状态检查等

# ---------- 检查 kubectl 可用 ----------
kubectl_check() {
    if ! kubectl get nodes >/dev/null 2>&1; then
        echo 'ERROR: kubectl 无法访问集群 (请先配置 ~/.kube/config)' >&2
        return 1
    fi
    return 0
}

# ---------- 登录 Rancher 并获取 kubeconfig ----------
rancher_login() {
    local rancher_url="$1"
    local rancher_user="$2"
    local rancher_pass="$3"

    local rancher_token
    rancher_token="$(curl -sk -X POST "$rancher_url/v3-public/localProviders/local?action=login" \
        -H 'Content-Type: application/json' \
        -d "{\"username\":\"$rancher_user\",\"password\":\"$rancher_pass\",\"responseType\":\"json\"}" \
        | python -c 'import json,sys; print(json.load(sys.stdin).get("token",""))' 2>/dev/null || true)"

    if [[ -z "$rancher_token" ]]; then
        echo 'ERROR: cannot get Rancher token (Invalid CSRF token 常见原因: Rancher 版本 CSRF 校验, 可先浏览器登录一次或检查 /v3-public 可访问性)' >&2
        return 1
    fi

    local kube_config
    kube_config="$(curl -sk -X POST "$rancher_url/v3/clusters/local?action=generateKubeconfig" \
        -H 'Content-Type: application/json' -H "Authorization: Bearer ***" -d '{}' \
        | python -c 'import json,sys; print(json.load(sys.stdin).get("config",""))' 2>/dev/null || true)"

    if [[ -z "$kube_config" ]]; then
        echo 'ERROR: cannot get kubeconfig from Rancher' >&2
        return 1
    fi

    local KUBE_DIR="$HOME/.kube"
    mkdir -p "$KUBE_DIR"
    echo "$kube_config" > "$KUBE_DIR/config"
    kubectl config set-cluster local --server="$rancher_url/k8s/clusters/local" --insecure-skip-tls-verify=true >/dev/null 2>&1 || true

    return 0
}

# ---------- 等待 Pod 就绪 ----------
wait_pods_ready() {
    local namespace="${1:?用法: wait_pods_ready <namespace>}"
    local timeout="${2:-120}"

    echo '======================== Result ========================'

    local ready=0
    local total=0

    while read -r _ status _; do
        if [[ "$status" =~ ^([0-9]+)/([0-9]+)$ ]]; then
            total=$((total + ${BASH_REMATCH[2]}))
            if [[ "${BASH_REMATCH[1]}" == "${BASH_REMATCH[2]}" ]]; then
                ready=$((ready + ${BASH_REMATCH[1]}))
            fi
        fi
    done < <(kubectl get pods -n "$namespace" -l app.kubernetes.io/instance=myai --no-headers 2>/dev/null || true)

    echo "Ready pods: $ready/$total"

    if [[ "$total" -gt 0 && "$ready" -eq "$total" ]]; then
        echo 'SUCCESS: All services ready!'
        return 0
    else
        echo 'WARNING: Some pods not ready' >&2
        return 1
    fi
}

# ---------- 清理异常 Pod ----------
cleanup_pods() {
    local namespace="${1:?用法: cleanup_pods <namespace>}"

    kubectl get pods -n "$namespace" -l app.kubernetes.io/instance=myai --no-headers 2>/dev/null | while read -r pod rest; do
        local state
        state="$(echo "$rest" | awk '{print $2}')"
        if [[ "$state" =~ ContainerCreating|CrashLoopBackOff|Terminating ]]; then
            kubectl delete pod "$pod" -n "$namespace" --grace-period=0 --force >/dev/null 2>&1 || true
        fi
    done
}
