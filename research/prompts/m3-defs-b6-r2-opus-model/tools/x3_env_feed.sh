#!/usr/bin/env bash
# X3 喂法：implementation-writer 在 <仓>/crates/singlefs-harness 里起的 env 各种写法，Bash 工具的 hook JSON 喂 heavy-test-guard.sh 入口，打印退出码与 stderr 首行。
# 用法：x3_env_feed.sh <被判仓根>
set -u
repo=$(cd "$1" && pwd)
cwd="$repo/crates/singlefs-harness"
W="bash ../../scripts/hosts/run-with-memory-cap.sh 2G"
feed() {
  local json err rc
  json=$(python3 -c 'import json,sys; print(json.dumps({"tool_name":"Bash","cwd":sys.argv[1],"agent_type":"implementation-writer","tool_input":{"command":sys.argv[2]}}))' "$cwd" "$2")
  err=$(printf '%s' "$json" | CLAUDE_PROJECT_DIR="$repo" bash "$repo/.claude/hooks/heavy-test-guard.sh" 2>&1 >/dev/null); rc=$?
  printf '%s\trc=%s\t%s\n' "$1" "$rc" "$(printf '%s' "$err" | grep -m1 -v '^\s*$' | cut -c1-200)"
}
feed E0-control-p "$W cargo test -p singlefs-checker-tier"
feed E1-prefix-var-in-S "PKG=singlefs-checker-tier $W env -S 'cargo test -p \${PKG}'"
feed E1b-export-var-in-S "export PKG=singlefs-checker-tier; $W env -S 'cargo test -p \${PKG}'"
feed E1c-prefix-var-part "TIER=tier $W env -S 'cargo test -p singlefs-checker-\${TIER}'"
feed E1d-prefix-var-chdir-in-S "D=../singlefs-checker-tier $W env -S '-C \${D} cargo test'"
feed E1e-plain-shell-var-p "PKG=singlefs-checker-tier; $W cargo test -p \"\$PKG\""
feed E1f-export-shell-var-p "export PKG=singlefs-checker-tier; $W cargo test -p \"\$PKG\""
feed E2-double-dash "$W env -- cargo test -p singlefs-checker-tier"
feed E3-unset "$W env -u FOO cargo test -p singlefs-checker-tier"
feed E4-long-eq-both "$W env --unset=FOO --chdir=../singlefs-checker-tier cargo test"
feed E5-abbrev-split "$W env --spl='cargo test -p singlefs-checker-tier'"
feed E6-nested-env-C "$W env -S 'env -C ../singlefs-checker-tier cargo test'"
feed E8-underscore-sep "$W env -S'cargo\\_test\\_-p\\_singlefs-checker-tier'"
feed E9-quoted-in-S "$W env -S 'cargo \"test\" -p \"singlefs-checker-tier\"'"
feed E10-trailing-args "$W env -S 'cargo test' -p singlefs-checker-tier"
feed E11-vS "$W env -vS 'cargo test -p singlefs-checker-tier'"
feed E12-comment-then-args "$W env -S 'cargo test #x' -p singlefs-checker-tier"
feed E13-u-value-looks-like-C "$W env -uC ../singlefs-checker-tier cargo test"
feed E14-default-signal "$W env --default-signal=PIPE -C ../singlefs-checker-tier cargo test"
