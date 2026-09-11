#!/usr/bin/env bash
# gate-stage: 脚本的执行位在暂存区里没有丢
#
# 手工只暂存「这一轮的」时，`git update-index --cacheinfo` 要自己写模式；写死 100644 就把可执行脚本的执行位抹了，
# 而工作区里那份还是可执行的——在工作区上跑的门禁一声不吭，丢掉的执行位就这样进了提交。
# 实测（2026-09-11）：dbd801d 把 research/scripts/replay.sh 从 100755 提交成 100644，下一次提交才发现并恢复。
#
# 判据（只看 .claude/gate.d、.claude/scripts、research/scripts 下已跟踪的 .sh 与 .py，不含 fixtures/）：
#   ① `.sh` 在暂存区里必须是 100755；
#   ② 暂存区里的模式与工作区的执行位一致（100755 ⇔ 工作区可执行）。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
git rev-parse --git-dir >/dev/null 2>&1 || { echo "  ! 不在 git 仓库里，本阶段跳过"; exit 0; }
checked=0; wrong_mode_count=0
while IFS= read -r -d '' entry; do
  mode="${entry%% *}"; path="${entry#*$'\t'}"
  case "$path" in */fixtures/*) continue ;; esac
  case "$path" in *.sh|*.py) ;; *) continue ;; esac
  checked=$((checked + 1))
  if [[ "$path" == *.sh && "$mode" != 100755 ]]; then
    echo "  ✗ $path 在暂存区里是 $mode，.sh 要可执行"   # gate-lint:detail
    wrong_mode_count=$((wrong_mode_count + 1)); continue
  fi
  [[ -f "$path" ]] || continue
  if { [[ -x "$path" ]] && [[ "$mode" != 100755 ]]; } || { [[ ! -x "$path" ]] && [[ "$mode" == 100755 ]]; }; then
    echo "  ✗ $path 在暂存区里是 $mode，与工作区的执行位不一致"   # gate-lint:detail
    wrong_mode_count=$((wrong_mode_count + 1))
  fi
done < <(git ls-files -s -z -- .claude/gate.d .claude/scripts research/scripts)
if (( checked == 0 )); then
  echo "  ✗ 一个脚本都没查到"
  echo "     → 在仓根跑；三处目录下没有已跟踪的 .sh / .py 时，这一条什么也没验。"
  exit 1
fi
if (( wrong_mode_count > 0 )); then
  echo "  ✗ $wrong_mode_count 个脚本的执行位在暂存区里不对（查了 $checked 个）"   # gate-lint:summary
  echo "     → 用 git update-index --chmod=+x <路径>（或 -x）改暂存区里的模式，让它与工作区一致；手工暂存别写死 100644。"
  exit 1
fi
echo "  ✓ 查了 $checked 个脚本：.sh 都可执行，暂存区里的模式与工作区一致"
