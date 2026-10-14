#!/usr/bin/env bash
# Y2 喂法：implementation-writer 在 <仓>/crates/singlefs-harness 里起的 env 组合写法，做成 Bash 工具的 hook JSON 喂 heavy-test-guard.sh 入口。
# 用法：y2_env_feed.sh <被判仓根>   输出：<格名>\trc=<退出码>\t<stderr 第一行非空行>
set -u
repo=$(cd "$1" && pwd)
cwd="$repo/crates/singlefs-harness"
W="bash ../../scripts/hosts/run-with-memory-cap.sh 2G"
feed() {
  local json err rc
  json=$(python3 -c 'import json,sys; print(json.dumps({"tool_name":"Bash","cwd":sys.argv[1],"agent_type":"implementation-writer","tool_input":{"command":sys.argv[2]}}))' "$cwd" "$2")
  err=$(printf '%s' "$json" | PYTHONDONTWRITEBYTECODE=1 CLAUDE_PROJECT_DIR="$repo" bash "$repo/.claude/hooks/heavy-test-guard.sh" 2>&1 >/dev/null); rc=$?
  printf '%s\trc=%s\t%s\n' "$1" "$rc" "$(printf '%s' "$err" | grep -m1 -v '^\s*$' | cut -c1-120)"
}
feed N0-control-env-C "$W env -C ../singlefs-checker-tier cargo test"
feed N1-nested-env-C "$W env -C .. env -C singlefs-checker-tier cargo test"
feed N1b-nested-env-C-abs-then-rel "$W env -C ../.. env -C crates/singlefs-checker-tier cargo test"
feed N2-env-S-nested-env-C "$W env -S '-C .. env -C singlefs-checker-tier cargo test'"
feed N3-env-C-then-sh-c-cd "$W env -C .. sh -c 'cd singlefs-checker-tier && cargo test'"
feed N4-cd-then-env-C-rel "cd .. && $W env -C singlefs-checker-tier cargo test"
feed N5-env-C-then-wrapper-rel "env -C ../singlefs-checker-tier bash ../../scripts/hosts/run-with-memory-cap.sh 2G cargo test"
feed N6-env-C-subdir-tests "$W env -C ../singlefs-checker-tier/tests cargo test"
feed N6b-cd-subdir-src "cd ../singlefs-checker-tier/src && $W cargo test"
feed N7-env-C-dotdot-normal "$W env -C ../singlefs-checker-tier/tests/.. cargo test"
feed N8-env-C-then-nice "$W env -C ../singlefs-checker-tier nice -n 19 cargo test"
feed N9-env-C-then-timeout "$W env -C ../singlefs-checker-tier timeout 600 cargo test"
feed N10-env-C-cargo-alias-t "$W env -C ../singlefs-checker-tier cargo t"
feed N11-env-C-cargo-toolchain "$W env -C ../singlefs-checker-tier cargo +stable test"
feed N12-env-C-manifest "$W env -C .. cargo test --manifest-path singlefs-checker-tier/Cargo.toml"
feed N13-nested-env-C-capped "$W env -C .. bash ../scripts/hosts/capped.sh 31 env -C singlefs-checker-tier cargo test"
feed N14-env-u-chdir-abbrev-eq "$W env --ch=../singlefs-checker-tier cargo test"
feed N15-env-C-then-bash-c "$W env -C ../singlefs-checker-tier bash -c 'cargo test'"
feed N16-env-C-dot-then-S "$W env -C .. -S 'env -C singlefs-checker-tier cargo test'"
