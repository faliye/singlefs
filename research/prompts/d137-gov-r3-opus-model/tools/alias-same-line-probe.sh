#!/usr/bin/env bash
# 件 16 放开的 E4（同一行先 alias 再用，这一行不展开别名）在 Claude Code 的 Bash 工具那种起法下真不真：
# 工具把命令包成 `bash -c "source <快照> … && eval '<命令>' && pwd -P …"`，快照里有 `shopt -s expand_aliases`。
# 样本只用 echo（不起 cargo、不造新写法）：分类报告 E4 那一行的形状，把 cargo 换成 echo。每条样本 setsid --wait 起、timeout 10 秒杀整组。
# 用法：alias-same-line-probe.sh <输出文件>；快照取 ~/.claude/shell-snapshots/ 下最新的一份 snapshot-bash-*.sh，没有就只跑不带快照的两格。
set -uo pipefail
out="$1"
snapshot="$(ls -t "$HOME"/.claude/shell-snapshots/snapshot-bash-*.sh 2>/dev/null | head -n 1)"
run_sample() {   # run_sample <标签> <bash -c 的代码>
  local label="$1" code="$2" text rc
  text="$(timeout -s KILL 10 setsid --wait bash -c "$code" 2>&1)"; rc=$?
  printf 'name=alias_same_line case=%s rc=%s expanded=%s\n  output: %s\n' "$label" "$rc" "$([[ "$text" == *ALIAS_EXPANDED* ]] && echo yes || echo no)" "$(printf '%s' "$text" | tr '\n' ' ' | cut -c1-200)" >> "$out"
}
: > "$out"
run_sample "plain-bash-c-same-line" "shopt -s expand_aliases; alias ct='echo ALIAS_EXPANDED'; ct"
run_sample "plain-bash-c-eval-same-line" "eval 'shopt -s expand_aliases; alias ct=\"echo ALIAS_EXPANDED\"; ct'"
if [[ -n "$snapshot" ]]; then
  run_sample "tool-wrapper-expand-aliases-state" "source '$snapshot' 2>/dev/null || true && shopt -u extglob 2>/dev/null || true && eval 'shopt expand_aliases'"
  run_sample "tool-wrapper-eval-same-line-with-shopt" "source '$snapshot' 2>/dev/null || true && shopt -u extglob 2>/dev/null || true && eval 'shopt -s expand_aliases; alias ct=\"echo ALIAS_EXPANDED\"; ct'"
  run_sample "tool-wrapper-eval-same-line-without-shopt" "source '$snapshot' 2>/dev/null || true && shopt -u extglob 2>/dev/null || true && eval 'alias ct=\"echo ALIAS_EXPANDED\"; ct'"
else
  echo "name=alias_same_line snapshot=none" >> "$out"
fi
echo "name=alias_same_line bash_version=${BASH_VERSION}" >> "$out"
