#!/usr/bin/env bash
# BASH_ENV + 转发函数绕过重型闸模型：从 <仓根> 拷一棵最小钩子树到临时目录，喂两条命令给 heavy-test-guard.sh。
# 直接 BASH_ENV=<起 checker 档测试包的脚本> bash -c true  期望拒 rc=2；f(){ bash -c true; }; BASH_ENV=<同一脚本> f  实测放行 rc=0 且不写检出。
# 用法：bash bash_env_forwarding_escape.sh <仓根> [<喂脚本 feed_hook.py>]。只喂钩子，不真执行里面的任何命令。
set -euo pipefail
repo=${1:?用法：bash bash_env_forwarding_escape.sh <仓根> [feed_hook.py]}
feed=${2:-"$(dirname "$0")/feed_hook.py"}
work=$(mktemp -d "${TMPDIR:-/tmp}/bash-env-escape.XXXXXX")
trap 'rm -rf "$work"' EXIT
tree="$work/tree"
mkdir -p "$tree/.claude/singlefs-ai-sop" "$tree/sub"
rsync -a --exclude __pycache__ "$repo/.claude/hooks" "$tree/.claude/"
rsync -a --exclude __pycache__ "$repo/.claude/scripts" "$tree/.claude/"
cp "$repo/.claude/heavy-thread-default" "$tree/.claude/"
rsync -a --exclude __pycache__ "$repo/.claude/singlefs-ai-sop/scripts" "$tree/.claude/singlefs-ai-sop/"
printf 'cargo test -p singlefs-checker-tier\n' > "$tree/sub/startup.sh"
export SINGLEFS_WORK_ROOT="$work/workroot"
printf '%s' 'BASH_ENV=sub/startup.sh bash -c true' > "$work/direct.txt"
printf '%s' 'f() { bash -c true; }; BASH_ENV=sub/startup.sh f' > "$work/forward.txt"
echo "name=bash_env_escape form=direct"
python3 -B "$feed" "$tree/.claude/hooks/heavy-test-guard.sh" "$tree" implementation-writer "$work/direct.txt" "$work/det-direct.jsonl" | head -1
python3 -B "$feed" "$tree/.claude/hooks/heavy-test-guard.sh" "$tree" implementation-writer "$work/forward.txt" "$work/det-fwd.jsonl" | head -1
echo "name=bash_env_escape form=forwarding-function detections_after=$(wc -l < "$work/det-fwd.jsonl" 2>/dev/null || echo 0)"
