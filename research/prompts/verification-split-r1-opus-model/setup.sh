#!/usr/bin/env bash
# 攻方腿（verification-split-r1）复跑用：把开工快照解到 $1（默认 /tmp/claude-1000/three-way-attack/verification-split-r1/snap），
# 再把主仓的规范副本 .claude/singlefs-ai-sop 拷进去（快照里没有它，heavy-test-guard.sh 与 94 号 source 它的 preflight.sh）。
set -euo pipefail
REPOSITORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
SNAPSHOT="${1:-/tmp/claude-1000/three-way-attack/verification-split-r1/snap}"
mkdir -p "$SNAPSHOT"
git -C "$REPOSITORY" archive refs/sop/verification-split-r1-snapshot | tar -x -C "$SNAPSHOT"
cp -r "$REPOSITORY/.claude/singlefs-ai-sop" "$SNAPSHOT/.claude/"
echo "snapshot at $SNAPSHOT"
