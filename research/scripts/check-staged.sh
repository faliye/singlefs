#!/usr/bin/env bash
# 只拿「HEAD + 暂存区」跑门禁阶段：几个会话共写一个仓时，工作区里混着别人没收尾的改动与未跟踪文件，
# 门禁在工作区上判的红分不清是谁的。这里在临时 worktree 上套 `git diff --cached`，别人的东西一样都不进来。
#
#   bash research/scripts/check-staged.sh              # 默认：doc-lint + 快的 kb 阶段（不含构建、复跑、外部引用）
#   bash research/scripts/check-staged.sh doc 34 60    # 只跑 doc-lint 与这几号阶段
#   bash research/scripts/check-staged.sh --selftest
#
# 规范副本 .claude/singlefs-ai-sop/ 不进 git，worktree 里没有它；有就原样拷进去，doc-lint 与链接检查才跑得起来。
# 自证会红：--selftest 在临时仓里放一个「文件里有 BAD 就红」的阶段，确认工作区里没暂存的 BAD 不算、暂存了的 BAD 判红；
# 再用 CHECK_STAGED_USE_WORKTREE=1 改成拿工作区原样去跑，确认没暂存的 BAD 也被算进来、selftest 判红；
# CHECK_STAGED_NO_TRAP=1 关掉打断时的清理，确认 selftest 判红（临时 worktree 留在仓里）。
set -uo pipefail

DEFAULT_STAGES=(doc 10 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 42 43 44 45 50 60 61 85 86)

# 跑到一半被打断（Ctrl-C）或被杀时也要删掉临时 worktree：留下的会一直登记在仓里，下一次还得手工 git worktree prune。
cleanup_isolated() {
  if [[ -n "${ISOLATED_WORKTREE:-}" ]]; then git -C "$ISOLATED_REPO" worktree remove --force "$ISOLATED_WORKTREE" >/dev/null 2>&1; fi
  if [[ -n "${ISOLATED_BASE:-}" ]]; then rm -rf "${ISOLATED_BASE:?}"; fi
  ISOLATED_WORKTREE=""; ISOLATED_BASE=""
}

run_isolated() {
  local repo="$1"; shift
  local stages=("$@")
  local base wt patch red=0 ran=0
  base="$(mktemp -d)"; wt="$base/wt"; patch="$base/staged.patch"
  git -C "$repo" diff --cached --binary > "$patch" || { echo "  ✗ 取不到暂存区的 diff"; echo "    → 在仓里跑，并确认 git 可用"; rm -rf "${base:?}"; return 2; }
  git -C "$repo" worktree add --detach "$wt" HEAD >/dev/null 2>&1 || { echo "  ✗ 建临时 worktree 失败"; echo "    → git worktree prune 之后再试"; rm -rf "${base:?}"; return 2; }
  ISOLATED_REPO="$repo"; ISOLATED_BASE="$base"; ISOLATED_WORKTREE="$wt"
  if [[ "${CHECK_STAGED_NO_TRAP:-0}" != 1 ]]; then
    trap 'cleanup_isolated; exit 130' INT
    trap 'cleanup_isolated; exit 143' TERM
  fi
  if [[ "${CHECK_STAGED_USE_WORKTREE:-0}" == 1 ]]; then
    (cd "$repo" && git ls-files -z) | while IFS= read -r -d '' tracked; do
      [[ -f "$repo/$tracked" ]] && mkdir -p "$wt/$(dirname "$tracked")" && cp "$repo/$tracked" "$wt/$tracked"
    done
  elif [[ -s "$patch" ]]; then
    git -C "$wt" apply --index "$patch" || { echo "  ✗ 暂存区的 diff 套不上 HEAD"; echo "    → 先 git status 看暂存区是不是基于当前 HEAD"; trap - INT TERM; cleanup_isolated; return 2; }
  fi
  if [[ -d "$repo/.claude/singlefs-ai-sop" && ! -d "$wt/.claude/singlefs-ai-sop" ]]; then
    cp -r "$repo/.claude/singlefs-ai-sop" "$wt/.claude/"
  fi
  local stage matched script out rc
  for stage in "${stages[@]}"; do
    if [[ "$stage" == doc ]]; then
      if [[ ! -f "$wt/.claude/singlefs-ai-sop/scripts/doc-lint.sh" ]]; then
        echo "  ! doc-lint 没跑：worktree 里没有规范副本（这一项没验，不是通过）"; continue
      fi
      out="$(cd "$wt" && bash .claude/singlefs-ai-sop/scripts/doc-lint.sh 2>&1)"; rc=$?; ran=$((ran + 1))
      if [[ $rc -eq 0 ]]; then echo "  ok  doc-lint"; else red=$((red + 1)); echo "  RED doc-lint"; echo "$out" | grep -E "✗|→" | head -8; fi
      continue
    fi
    matched=0
    for script in "$wt"/.claude/gate.d/"$stage"-*.sh; do
      [[ -f "$script" ]] || continue
      matched=1; ran=$((ran + 1))
      out="$(cd "$wt" && bash "$script" 2>&1)"; rc=$?
      if [[ $rc -eq 0 ]]; then echo "  ok  $(basename "$script")"; else red=$((red + 1)); echo "  RED $(basename "$script")"; echo "$out" | grep -E "✗|→" | head -8; fi
    done
    [[ $matched -eq 1 ]] || echo "  ! 阶段号 $stage 没有对应的 .claude/gate.d/$stage-*.sh（没跑）"
  done
  trap - INT TERM
  cleanup_isolated
  if [[ $ran -eq 0 ]]; then
    echo "  ✗ 一个阶段都没跑到"
    echo "    → 阶段号写成 .claude/gate.d/ 下文件名的前缀（例：34），doc-lint 写 doc"
    return 1
  fi
  if [[ $red -gt 0 ]]; then
    echo "  ✗ HEAD + 暂存区上跑了 $ran 个阶段，红 $red 个"
    echo "    → 红的是这一次提交带进来的（别人的未提交改动不在这棵树里）；先查它在不在 HEAD 上就红：暂存区清空再跑一次"
    return 1
  fi
  echo "  ✓ HEAD + 暂存区上跑了 $ran 个阶段，全绿"
  return 0
}

selftest() {
  local repo rc_clean rc_bad
  repo="$(mktemp -d)"
  git -C "$repo" init -q && git -C "$repo" config user.email selftest@example.invalid && git -C "$repo" config user.name selftest
  mkdir -p "$repo/.claude/gate.d" "$repo/kb"
  printf '#!/usr/bin/env bash\n# gate-stage: selftest\nif grep -rq BAD kb/; then echo "  ✗ kb 里有 BAD"; echo "    → 删掉"; exit 1; fi\nexit 0\n' > "$repo/.claude/gate.d/20-selftest.sh"
  echo "干净" > "$repo/kb/a.md"; echo "干净" > "$repo/kb/b.md"
  git -C "$repo" add -A && git -C "$repo" commit -q -m base
  echo "BAD（别人没收尾的）" >> "$repo/kb/a.md"
  echo "我的" >> "$repo/kb/b.md" && git -C "$repo" add kb/b.md
  run_isolated "$repo" 20 >/dev/null; rc_clean=$?
  if [[ "${CHECK_STAGED_USE_WORKTREE:-0}" == 1 ]]; then
    rm -rf "${repo:?}"
    if [[ $rc_clean -eq 0 ]]; then echo "selftest: 拿工作区原样去跑，没暂存的 BAD 却没算进来 —— 检查坏了"; return 1; fi
    echo "selftest: 拿工作区原样去跑确认判红（没暂存的 BAD 被算进来）"; return 0
  fi
  echo "BAD（我的）" >> "$repo/kb/b.md" && git -C "$repo" add kb/b.md
  run_isolated "$repo" 20 >/dev/null; rc_bad=$?
  # 跑到一半被打断：一个睡 20 秒的阶段，worktree 建起来之后给整组发 INT（Ctrl-C 的形态）；
  # 打断之后仓里只许剩主工作区那一个登记。开 job control（set -m）是为了让后台那一组收得到 INT。
  printf '#!/usr/bin/env bash\n# gate-stage: selftest-slow\nsleep 20\n' > "$repo/.claude/gate.d/30-slow.sh"
  git -C "$repo" add .claude/gate.d/30-slow.sh
  local registered_after_interrupt leftover
  registered_after_interrupt="$(
    set -m
    ( run_isolated "$repo" 30 >/dev/null 2>&1 ) &
    job="$!"
    for _ in $(seq 1 100); do
      [[ "$(git -C "$repo" worktree list --porcelain | grep -c '^worktree ')" -ge 2 ]] && break
      sleep 0.1
    done
    kill -INT -- -"$job" 2>/dev/null; wait "$job" 2>/dev/null
    git -C "$repo" worktree list --porcelain | grep -c '^worktree '
  )"
  while IFS= read -r leftover; do [[ -n "$leftover" ]] && rm -rf "${leftover:?}"; done \
    < <(git -C "$repo" worktree list --porcelain | sed -n 's/^worktree //p' | tail -n +2)
  rm -rf "${repo:?}"
  if [[ "${CHECK_STAGED_NO_TRAP:-0}" == 1 ]]; then
    if [[ "$registered_after_interrupt" == 1 ]]; then echo "selftest: 关掉 trap 之后打断仍然没留下 worktree —— 检查坏了"; return 1; fi
    echo "selftest: 关掉 trap 确认判红（打断之后临时 worktree 留在仓里）"; return 0
  fi
  if [[ "$registered_after_interrupt" != 1 ]]; then echo "selftest: 跑到一半被打断，临时 worktree 留在仓里（登记 $registered_after_interrupt 个）"; return 1; fi
  if [[ $rc_clean -ne 0 ]]; then echo "selftest: 只有工作区里没暂存的 BAD，却判红了 —— 别人的改动漏进来了"; return 1; fi
  if [[ $rc_bad -ne 1 ]]; then echo "selftest: 暂存了 BAD 却没判红"; return 1; fi
  echo "selftest: 通过（没暂存的 BAD 不算、暂存了的 BAD 判红、跑到一半被打断也清掉临时 worktree）"
  return 0
}

if [[ "${1:-}" == --selftest ]]; then
  selftest; exit $?
fi
repo_root="$(git rev-parse --show-toplevel)" || { echo "  ✗ 不在 git 仓里"; echo "    → cd 到仓里再跑"; exit 2; }
if [[ $# -gt 0 ]]; then run_isolated "$repo_root" "$@"; else run_isolated "$repo_root" "${DEFAULT_STAGES[@]}"; fi
