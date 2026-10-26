#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git
# 跑 `gate.sh --staged`，整轮全绿、调用方又没给 GATE_BASE 才把 `refs/sop/staged-green` 前移到这一次被判的那棵暂存树。
# ref 留着只作记录（「上次整轮全绿的树」，别处要拿它当改动基准的照读）：任何复用判定都不读它，复用只看每条判定的单元键（scripts/gate/admission.py 文件头）。
# 交给 gate.sh 的两样：被判的暂存树另经 GATE_STAGED_TREE 交给上游（上游在 worktree 里 read-tree 这一棵，不自己重算）；
# 编译目录 CARGO_TARGET_DIR（用户 2026-10-23 定案「自己用完自己删」，不跨趟复用）：调用方给了 SINGLEFS_GATE_TARGET_DIR 就照它、收尾不删；
# 没给就在这一趟自己的目录 <日志目录>/gate-staged-<日期时刻>-<暂存树前 12 位>-<进程号>/ 里建 target，正常结束、红了、被 TERM / INT 停（等 gate.sh 退出）
# 都在 EXIT 上删（scripts/hosts/run-build-directory.sh：删之前核本机没有进程开着里面的文件，有就不删、报出进程、退非 0），再删掉那个空目录。
#
#   gate-staged.sh [项目根] [传给 gate.sh 的别的参数…]
#   gate-staged.sh --light [项目根]   轻档：只跑不重的门禁（见下「轻档」），重型几道报本次未跑；不前移 ref、不跑 gate.sh
#   gate-staged.sh --selftest   自证：转给 scripts/gate/gate-staged-selftest.sh（临时仓 + 假 gate.sh，全绿 / 有红 / 跑的过程中暂存区变了 / 给了 GATE_BASE，格数由它的成功行现算）
#
# 为什么前移放在外壳里：共享 `gate.sh` 在上游（`.claude/singlefs-ai-sop/scripts/`），本仓的
# `.claude/scripts/gate.sh` 只是 `exec` 转发，中间没有钩子。前移那一句只能由**起门禁的人**来做。
#
# 三条规矩：
#   ① 只认暂存树：`git write-tree` 算出来的那一棵，别的会话未暂存的改动进不来（几个会话共写一个仓时，工作区不代表任何人打算提交的东西）。
#   ② 只有整轮**全绿**才前移；有红不动——红的那一趟什么也没证明。调用方给了 GATE_BASE 也不动——那一趟只判了起点之后的改动。
#   ③ 不带 `--staged` 跑不许前移，所以这个外壳只做 `--staged` 这一条路。
# ref 是覆盖语义，永远只有一条，不累积、不落盘第二份状态。
#
# 轻档（--light；.claude/kb/milestone/03-third-txn.md「待优化」第 56、48 行：只想在暂存树上跑不重的门禁，整轮跳不掉刚跑过的重型几道）：
#   被判的还是同一棵暂存树（git write-tree），交给 scripts/gate/staged-tree-worktree.sh light-stages：它在那棵树的私有 worktree 上（被忽略的规范副本与本地配置拷进去）
#   并行跑门禁目录里不重的每一道（重型照 .claude/hooks/lib_heavy_tests.py 的 STAGE_KIND 认，逐道报本次未跑），每道的输出留在
#   <日志目录>/gate-staged-…-light/<阶段>.log；退出码转手（0 没有红，1 有红）。不经 gate.sh，上游共享的几道 lint 也不跑；refs/sop/staged-green 一律不前移。
#   轻档的自证在 staged-tree-worktree.sh --selftest 里（「gate-staged.sh --light 跑轻档、不前移、退出码转手」那一格）。
#
# 日志出两份（用户 2026-10-21：整轮门禁的输出动辄上万行，一般只看错误警告）：gate.sh 的输出照常转打，同时经 scripts/hosts/two-logs.sh 写
#   <日志目录>/gate-staged-<日期时刻>-<暂存树前 12 位>-<进程号>.log（全量）与同名的 .problems.log（只收 ✗ 与它的 → 出路、⚠、本次未跑、逐格汇总、
#   cargo / rustc 的 error 与 warning:、libtest 的 FAILED 与 panicked、Traceback、非 0 退出码，行前带全量日志里的行号，末行写收了几行全量几行）。
#   日志目录取 SINGLEFS_GATE_STAGED_LOG_DIRECTORY，没给是 ~/.local/share/singlefs/gate-staged-logs；开跑前与收尾都打这两份的路径，红的出路先指到 .problems.log。
#   日志只增不删：太多了自己清（按文件名里的日期）。退出码仍是 gate.sh 的。
set -uo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" || exit 2
light_tier=0
if [[ "${1:-}" == "--light" ]]; then light_tier=1; shift; fi
ROOT="${1:-$(cd "$SCRIPT_DIRECTORY/../.." && pwd)}"
[[ "${1:-}" != "--selftest" ]] || exec bash "$SCRIPT_DIRECTORY/gate-staged-selftest.sh"
shift 2>/dev/null || true
cd "$ROOT" || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给仓库根的绝对路径，或者在仓里直接跑它。"; exit 2; }
ROOT="$PWD"

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ✗ $ROOT 不是 git 工作树"; echo "     → 怎么办：在这个项目的 git 仓里跑它；门禁的复用判定靠 git 的树对象。"; exit 2; }

staged_tree="$(git write-tree)" || { echo "  ✗ 算不出暂存树（git write-tree 失败）"; echo "     → 怎么办：先看 git status；索引坏了就 git read-tree HEAD 重建，再把这一批重新暂存。"; exit 2; }
echo "  这一趟被判的暂存树：${staged_tree:0:12}（HEAD + 暂存区；别的会话未暂存的改动不在里面）"

SINGLEFS_STAGED_TREE="$staged_tree"
export SINGLEFS_STAGED_TREE
GATE_STAGED_TREE="$staged_tree"
export GATE_STAGED_TREE
gate_log_directory="${SINGLEFS_GATE_STAGED_LOG_DIRECTORY:-$HOME/.local/share/singlefs/gate-staged-logs}"
gate_log_name="$gate_log_directory/gate-staged-$(date -u +%Y%m%dT%H%M%SZ)-${staged_tree:0:12}-$$"
# 编译目录（文件头）：调用方给了照它；没给建在这一趟自己的目录里，收尾的 EXIT trap 删
source "$SCRIPT_DIRECTORY/../hosts/run-build-directory.sh"
CARGO_TARGET_DIR="${SINGLEFS_GATE_TARGET_DIR:-}"
if [[ -n "$CARGO_TARGET_DIR" ]]; then
  export CARGO_TARGET_DIR
  echo "  编译目录：$CARGO_TARGET_DIR（调用方给的 SINGLEFS_GATE_TARGET_DIR，收尾不删）"
else
  unset CARGO_TARGET_DIR
  run_build_directory_begin "$gate_log_name" || exit 2
  gate_staged_finish() {
    local finish_exit=$?
    trap - EXIT
    run_build_directory_end || { ((finish_exit)) || finish_exit=1; }
    rmdir -- "$gate_log_name" 2>/dev/null
    exit "$finish_exit"
  }
  trap gate_staged_finish EXIT
  trap 'exit 143' TERM
  trap 'exit 130' INT
  echo "  这一趟的编译目录：$CARGO_TARGET_DIR（这里建的，收尾删）"
fi
two_logs_script="$SCRIPT_DIRECTORY/../hosts/two-logs.sh"
[[ -f "$two_logs_script" ]] || { echo "  ✗ 找不到 $two_logs_script（日志出两份的包装）"; echo "     → 怎么办：从完整的仓里跑它；scripts/hosts/two-logs.sh 随仓走。"; exit 2; }
if ((light_tier)); then
  if (($#)); then echo "  ✗ 轻档不收别的参数（收到：$*）"; echo "     → 怎么办：写成 gate-staged.sh --light [项目根]；要给 gate.sh 传参数就不带 --light 跑整轮。"; exit 2; fi
  echo "  轻档：只跑不重的门禁，每道的输出在 $gate_log_name-light/"
  if bash "$SCRIPT_DIRECTORY/staged-tree-worktree.sh" light-stages "$ROOT" --tree "$staged_tree" --log-directory "$gate_log_name-light"; then light_rc=0; else light_rc=$?; fi
  echo "  ! 轻档不前移 refs/sop/staged-green：重型几道与上游共享的 lint 没跑，这一趟不是整轮"
  exit "$light_rc"
fi
echo "  日志两份（跑完在这里；一般只看 .problems.log）：全量 $gate_log_name.log；错误警告 $gate_log_name.problems.log"
bash "$two_logs_script" "$gate_log_name" -- bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"
gate_rc=$?

if [[ $gate_rc -eq 0 && -n "${GATE_BASE:-}" ]]; then
  echo "  ! 调用方给了 GATE_BASE（${GATE_BASE}）：这一趟只判了那个起点之后的改动，refs/sop/staged-green 不前移"
  echo "     → 怎么办：要前移就不设 GATE_BASE 再跑一趟。"
elif [[ $gate_rc -eq 0 ]]; then
  # 前移之前再算一次：跑的这段时间里别人可能往暂存区放了东西，那时被判的那棵树已经不是现在这棵
  now_tree="$(git write-tree)"
  if [[ "$now_tree" != "$staged_tree" ]]; then
    echo "  ! 跑的过程中暂存区变了（${staged_tree:0:12} → ${now_tree:0:12}）：refs/sop/staged-green 不前移"
    echo "     → 怎么办：绿的是开跑时那一棵，现在这一棵没人验过。等暂存区停下来再跑一趟，或者直接提交开跑时那一批。"
  else
    # ref 指向「包着那棵暂存树的提交」，不是裸树（拿它当改动基准的按 <ref>^{commit} 取）；仍然只有一条 ref、不落盘第二份状态。
    green_commit="$(git commit-tree "$staged_tree" -m "gate --staged 整轮全绿：这棵暂存树上每一道都绿过")" || {
      echo "  ✗ 门禁通过，但创建全绿记录提交失败，refs/sop/staged-green 没有前移"
      echo "     → 怎么办：按 git 报错检查作者配置与对象库写权限，修好后重跑"
      exit 1
    }
    git update-ref refs/sop/staged-green "$green_commit" || {
      echo "  ✗ 门禁通过，但更新 refs/sop/staged-green 失败"
      echo "     → 怎么办：按 git 报错检查引用锁与写权限，修好后重跑；这次不能报告记录已前移"
      exit 1
    }
    echo "  ✓ 整轮全绿，refs/sop/staged-green 前移到 ${green_commit:0:12}（包着暂存树 ${staged_tree:0:12}）：只作「上次整轮全绿的树」的记录，复用判定不读它"
  fi
else
  echo "  ! 整轮没全绿，refs/sop/staged-green 不动：红的这一趟什么也没证明"
  echo "     → 怎么办：先看错误警告日志 $gate_log_name.problems.log（✗ 与它的 → 出路、⚠、本次未跑、逐格汇总都在里面，行前带全量日志里的行号）；全量在 $gate_log_name.log"
fi
exit $gate_rc
