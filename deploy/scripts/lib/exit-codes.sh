#!/bin/bash
# =============================================================================
# exit-codes.sh — 统一退出码定义
# =============================================================================
# 所有脚本使用此文件中的退出码定义，确保一致性
# =============================================================================

readonly EXIT_SUCCESS=0
readonly EXIT_FAILURE=1
readonly EXIT_PARSE_ERROR=2
readonly EXIT_LOGIN_ERROR=3
readonly EXIT_PUSH_ERROR=4
readonly EXIT_DEPLOY_ERROR=5
readonly EXIT_RESOURCE_ERROR=6
