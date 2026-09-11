#!/usr/bin/env bash
# PreToolUse hook（Write）：仓里已存在、又没进 git 的文件，拒绝用 Write 整份覆盖。
#
# 为什么：几个会话同时在一个仓里干活。2026-09-11 一个会话用 Write 新建
# `research/prompts/e137-preregistration.md`，另一个会话几分钟前刚写了同名的未提交文件，
# Write 回报「updated」就把它盖了——没进过 git，事后只能从那个会话的对话记录里重放 heredoc 复原。
# 写一句「新建文件先查一下」拦不住，工具层当场拒绝才拦得住（singlefs-ai-sop/rules/show-me-test.md）。
#
# 放行：目标不存在；目标已被 git 跟踪（覆盖了还能从 git 找回）；目标在仓外（scratchpad 之类）。
# 拒绝时退出码 2，stderr 交给模型：换一个没被占的名字排他新建，或者确认是这一轮自己的文件后用 Edit 改。
#
#   refuse-overwrite-untracked.sh            # 从 stdin 读 hook 的 JSON
#   refuse-overwrite-untracked.sh --selftest # 在临时仓里走四种情形；REFUSE_OVERWRITE_DISABLE_CHECK=1 时自检必须判红
set -uo pipefail

decide() {   # $1 = 目标路径；返回 0 放行、2 拒绝
  local target="$1" root
  [[ -e "$target" ]] || return 0
  root="$(git -C "$(dirname "$target")" rev-parse --show-toplevel 2>/dev/null)" || return 0
  [[ "${REFUSE_OVERWRITE_DISABLE_CHECK:-0}" == "1" ]] && return 0
  if git -C "$root" ls-files --error-unmatch -- "$target" >/dev/null 2>&1; then
    return 0
  fi
  return 2
}

selftest() {
  local tmp rc failures=0
  tmp="$(mktemp -d)"
  git -C "$tmp" init -q && git -C "$tmp" config user.email t@t && git -C "$tmp" config user.name t
  echo a > "$tmp/tracked.md" && git -C "$tmp" add tracked.md && git -C "$tmp" commit -qm t
  echo b > "$tmp/untracked.md"
  local outside; outside="$(mktemp)"
  for case in "untracked.md:2" "tracked.md:0" "missing.md:0" "OUTSIDE:0"; do
    local name="${case%%:*}" want="${case##*:}" path
    if [[ "$name" == OUTSIDE ]]; then path="$outside"; else path="$tmp/$name"; fi
    decide "$path"; rc=$?
    if [[ "$rc" != "$want" ]]; then
      echo "  ✗ 自检：$name 应当返回 $want，实际 $rc"
      echo "    → 看 decide() 的放行条件；REFUSE_OVERWRITE_DISABLE_CHECK 设着的话这里本来就该红"
      failures=$((failures + 1))
    fi
  done
  rm -rf "${tmp:?}" "$outside"
  if ((failures)); then return 1; fi
  echo "  ✓ 自检通过：未跟踪的已有文件拒绝，已跟踪 / 不存在 / 仓外放行（查了 4 种情形）"
}

if [[ "${1:-}" == "--selftest" ]]; then
  selftest; exit $?
fi

input="$(cat)"
target="$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print((data.get("tool_input") or {}).get("file_path",""))' "$input" 2>/dev/null)"
[[ -n "$target" ]] || exit 0
decide "$target"; rc=$?
if [[ $rc -eq 2 ]]; then
  echo "✗ 拒绝用 Write 整份覆盖 $target：它已存在而且没进 git，可能是别的会话还没提交的文件，盖掉就找不回来。" >&2
  echo "→ 新建文件：换一个没被占的名字，用排他方式写（set -o noclobber; cat > 路径 <<'EOF'，或 python open(路径, 'x')）。" >&2
  echo "→ 确认是这一轮自己的文件：先 Read 看一眼，再用 Edit 改；整份重写就先 git add 它。" >&2
  exit 2
fi
exit 0
