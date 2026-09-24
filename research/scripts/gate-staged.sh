#!/usr/bin/env bash
# 跑 `gate.sh --staged`，整轮全绿才把 `refs/sop/staged-green` 前移到这一次被判的那棵暂存树。
#
#   gate-staged.sh [项目根] [传给 gate.sh 的别的参数…]
#   gate-staged.sh --selftest   自证：转给 research/scripts/gate-staged-selftest.sh（临时仓 + 假 gate.sh，全绿 / 有红 / 跑的过程中暂存区变了，格数由它的成功行现算）
#
# 为什么前移放在外壳里：共享 `gate.sh` 在上游（`.claude/singlefs-ai-sop/scripts/`），本仓的
# `.claude/scripts/gate.sh` 只是 `exec` 转发，中间没有钩子。前移那一句只能由**起门禁的人**来做。
#
# 三条规矩（`research/scripts/stage-must-run.sh` 的文件头写了为什么）：
#   ① 只认暂存树：`git write-tree` 算出来的那一棵，别的会话未暂存的改动进不来。
#   ② 只有整轮**全绿**才前移；有红不动——红的那一趟什么也没证明。
#   ③ 不带 `--staged` 跑不许前移，所以这个外壳只做 `--staged` 这一条路。
# ref 是覆盖语义，永远只有一条，不累积、不落盘第二份状态。
set -uo pipefail

ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
[[ "${1:-}" != "--selftest" ]] || exec bash "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/gate-staged-selftest.sh"
shift 2>/dev/null || true
cd "$ROOT" || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给仓库根的绝对路径，或者在仓里直接跑它。"; exit 2; }

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ✗ $ROOT 不是 git 工作树"; echo "     → 怎么办：在这个项目的 git 仓里跑它；门禁的复用判定靠 git 的树对象。"; exit 2; }

staged_tree="$(git write-tree)" || { echo "  ✗ 算不出暂存树（git write-tree 失败）"; echo "     → 怎么办：先看 git status；索引坏了就 git read-tree HEAD 重建，再把这一批重新暂存。"; exit 2; }
echo "  这一趟被判的暂存树：${staged_tree:0:12}（HEAD + 暂存区；别的会话未暂存的改动不在里面）"

SINGLEFS_STAGED_TREE="$staged_tree"
export SINGLEFS_STAGED_TREE
bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"
gate_rc=$?

if [[ $gate_rc -eq 0 ]]; then
  # 前移之前再算一次：跑的这段时间里别人可能往暂存区放了东西，那时被判的那棵树已经不是现在这棵
  now_tree="$(git write-tree)"
  if [[ "$now_tree" != "$staged_tree" ]]; then
    echo "  ! 跑的过程中暂存区变了（${staged_tree:0:12} → ${now_tree:0:12}）：refs/sop/staged-green 不前移"
    echo "     → 怎么办：绿的是开跑时那一棵，现在这一棵没人验过。等暂存区停下来再跑一趟，或者直接提交开跑时那一批。"
  else
    # ref 指向「包着那棵暂存树的提交」，不是裸树：`git update-ref` 对 refs/heads/ 之外默认不写 reflog，
    # 裸树也读不出时刻，复用上限那一条就永远不触发（2026-09-22 实测，第一版栽在这里）。
    # 包一层提交，时刻从提交自己身上读，仍然只有一条 ref、不落盘第二份状态。
    green_commit="$(git commit-tree "$staged_tree" -m "gate --staged 整轮全绿：这棵暂存树上每一道都绿过")"
    git update-ref refs/sop/staged-green "$green_commit"
    echo "  ✓ 整轮全绿，refs/sop/staged-green 前移到 ${green_commit:0:12}（包着暂存树 ${staged_tree:0:12}）：下一趟里输入没变的阶段可以复用这次的判定"
  fi
else
  echo "  ! 整轮没全绿，refs/sop/staged-green 不动：红的这一趟什么也没证明"
fi
exit $gate_rc
