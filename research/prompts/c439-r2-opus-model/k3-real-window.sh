#!/usr/bin/env bash
# K3 的代价一面：本仓 staged-green 此刻停在哪、庚二下这一格此刻的窗口与红行（真跑抽出来的格判法，在 --shared 克隆上）。
# 用法：k3-real-window.sh <冻结副本> <工具副本> <草稿根> <克隆根> <staged-green 提交>
source "$(dirname "$0")/lib.sh" "$1" "$2" "$3"
CLONE="$4"; SG="$5"
echo "name=k3_real head=$(git -C "$CLONE" rev-parse --short=12 HEAD) staged_green=${SG:0:12} staged_green_date=$(git -C "$CLONE" log -1 --format=%cd --date=short "$SG") commits_since=$(git -C "$CLONE" rev-list --count HEAD --since="$(git -C "$CLONE" log -1 --format=%cI "$SG")")"
run_cell "$CLONE" "$SG" | sed -n '1p;2p;$p'
