#!/usr/bin/env bash
# V5 换目录喂法：implementation-writer 在 <仓>/crates/singlefs-harness 里起的各种换目录写法，喂 heavy-test-guard.sh 入口。
# 用法：v5_chdir_feed.sh <被判仓根>
set -u
repo=$(cd "$1" && pwd)
cwd="$repo/crates/singlefs-harness"
W="bash ../../scripts/hosts/run-with-memory-cap.sh 8G"
feed() {
  local json err rc
  json=$(python3 -c 'import json,sys; print(json.dumps({"tool_name":"Bash","cwd":sys.argv[1],"agent_type":"implementation-writer","tool_input":{"command":sys.argv[2]}}))' "$cwd" "$2")
  err=$(printf '%s' "$json" | CLAUDE_PROJECT_DIR="$repo" bash "$repo/.claude/hooks/heavy-test-guard.sh" 2>&1 >/dev/null); rc=$?
  printf '%s\trc=%s\t%s\n' "$1" "$rc" "$(printf '%s' "$err" | grep -m1 -v '^\s*$')"
}
feed C0-baseline-cd "cd ../singlefs-checker-tier && $W cargo test"
feed C1-env-C "$W env -C ../singlefs-checker-tier cargo test"
feed C2-env-iC-combined "$W env -iC ../singlefs-checker-tier cargo test"
feed C3-env-u-then-C "$W env -u RUSTFLAGS -C ../singlefs-checker-tier cargo test"
feed C4-pushd "pushd ../singlefs-checker-tier && $W cargo test"
feed C5-subshell-cd "( cd ../singlefs-checker-tier; $W cargo test )"
feed C6-env-C-var "D=../singlefs-checker-tier; $W env -C \"\$D\" cargo test"
feed C7-cargo-C-nightly "$W cargo -Zunstable-options -C ../singlefs-checker-tier test"
feed C8-env-S-split "$W env -S 'cargo test -p singlefs-checker-tier'"
feed C9-env-C-then-sh-c "$W env -C ../singlefs-checker-tier sh -c 'cargo test'"
feed C10-env-C-long-eq "$W env --chdir=../singlefs-checker-tier cargo test"
