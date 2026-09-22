#!/usr/bin/env bash
# 这一道阶段能不能复用上一次的判定：**要跑退 0，可跳过退 1，判不出来退 0**（与 change-touches-crates.sh 同极性）。
#
#   stage-must-run.sh <项目根> <阶段文件名>    判一次，把判据与依据打到 stdout
#   stage-must-run.sh --selftest               自证 8 格
#
# 名字说的是哪一边为真：**要不要跑**。退 0 = 要跑（输入变了、判不出来、或不许复用），退 1 = 可跳过。
# 与 change-touches-crates.sh 同极性（那边 0 = 碰了 = 要跑），接法也一样，阶段里照抄那四行就行。
#
# 判据只有一句：**这一道读的那几条路径，在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间，git 说变没变。**
# 路径清单在 `.claude/gate.d/stage-inputs.tsv`，那是唯一登记位；清单自己也进比对（清单变了必定重跑）。
#
# 为什么用树不用提交、为什么只一条 ref：
#   `refs/sop/gate-ok` 断言的是「**不带 --staged 的整轮**在哪个**提交**上绿过」，用来算 diff 窗口；
#   带 `--staged` 跑绿从不让它前移（窗口是人为收窄的，C449（两套基准算法让带不带 --staged 判出不同的改动范围）
#   与 C454（--staged 跑绿不让基准前移，而且不说）记的都是这一格）。
#   `refs/sop/staged-green` 断言的只有一句「**带 --staged 的整轮**在这棵**树**上绿过」，而 `--staged` 恰好完整验了这一句。
#   整轮全绿意味着每一道都在那棵树上绿过 ⇒ **一条 ref 就够**，不必按阶段各存一条；ref 是覆盖语义，存不下第二条。
#
# 为什么只认暂存树，不看工作区：
#   几个会话共写一个仓时，工作区随时被别人改，它不代表任何人打算提交的东西
#   （共享 gate.sh 的「跑的过程中工作区变没变」那道指纹检查立的就是这个理由）。
#   所以这一道只在 `SINGLEFS_STAGED_TREE` 给了树对象时才许复用——那个值由 research/scripts/gate-staged.sh 算一次、
#   传给整轮、跑绿之后前移同一棵。没有这个变量（有人直接跑 gate.sh）一律当成要跑。
#
# 管不到的：
#   **环境变了而内容没变**——rustc 升级、系统库换了，树一模一样而结论已经不作数。
#   把版本串纳入比对要么落盘、要么塞进 ref 名，两样都跟「只一条 ref、不落盘」冲突；
#   这里用**复用上限**替代：绿判定超过 SINGLEFS_REUSE_HOURS 小时（默认 24）就强制跑一趟并重新前移。
#   `Cargo.lock` 在各阶段的清单里，依赖变化看得见；工具链升级看不见，靠上限兜。
#   `SINGLEFS_GATE_FULL=1` 强制当成要跑。
set -uo pipefail

REUSE_REF="refs/sop/staged-green"
INPUT_TABLE=".claude/gate.d/stage-inputs.tsv"

say_run() { printf '%s\n' "$1"; exit 0; }   # 要跑
say_skip() { printf '%s\n' "$1"; exit 1; }  # 可跳过

if [[ "${1:-}" == "--selftest" ]]; then
  exec bash "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/stage-must-run-selftest.sh"
fi

ROOT="${1:-}"
STAGE="${2:-}"
[[ -n "$ROOT" && -n "$STAGE" ]] || { echo "  ✗ 用法：stage-must-run.sh <项目根> <阶段文件名>"; echo "     → 怎么办：阶段名照 .claude/gate.d/ 下的文件名写，例 59-crates-mutation-replay.sh。"; exit 0; }

[[ "${SINGLEFS_GATE_FULL:-}" != "1" ]] || say_run "强制全跑（SINGLEFS_GATE_FULL=1）"
git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1 || say_run "判不出来：$ROOT 不是 git 工作树，按要跑处理"

staged_tree="${SINGLEFS_STAGED_TREE:-}"
[[ -n "$staged_tree" ]] || say_run "这一趟不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE），被判的可能是工作区，不许复用"
git -C "$ROOT" rev-parse -q --verify "${staged_tree}^{tree}" >/dev/null 2>&1 || say_run "判不出来：SINGLEFS_STAGED_TREE=${staged_tree} 不是这个仓里的树对象"

green="$(git -C "$ROOT" rev-parse -q --verify "${REUSE_REF}^{commit}" 2>/dev/null || true)"
[[ -n "$green" ]] || say_run "还没有过整轮全绿的暂存树（$REUSE_REF 不存在）"

[[ -f "$ROOT/$INPUT_TABLE" ]] || say_run "读不到路径清单 $INPUT_TABLE，按要跑处理"
inputs="$(awk -F'\t' -v stage="$STAGE" '$1 == stage { print $2 }' "$ROOT/$INPUT_TABLE")"
[[ -n "$inputs" ]] || say_run "$INPUT_TABLE 里没有 $STAGE 这一行，按要跑处理"

# 清单自己进比对：少写一条输入，那条输入就永远不会让这道阶段重跑
read -r -a input_paths <<< "$inputs"
input_paths+=("$INPUT_TABLE")

hours="${SINGLEFS_REUSE_HOURS:-24}"
# 时刻读那个提交对象自己的。ref 指的是「包着那棵暂存树的提交」而不是裸树，正是为了这个：
# `git update-ref` 对 refs/heads/ 之外的 ref 默认不写 reflog，而裸树 `git log -g` 也读不了
# ——2026-09-22 实测，第一版靠 reflog 取时刻，上限那一条**一次都没触发过**。
green_epoch="$(git -C "$ROOT" log -1 --format=%ct "$green" 2>/dev/null || true)"
[[ "$green_epoch" =~ ^[0-9]+$ ]] || say_run "取不到 $REUSE_REF 的时刻（它不是包着暂存树的提交？），判不出复用上限，按要跑处理"
age_hours=$(( ( $(date +%s) - green_epoch ) / 3600 ))
if (( age_hours >= hours )); then
  say_run "上次整轮全绿在 ${age_hours} 小时前，到了复用上限 ${hours} 小时：内容没变不代表环境没变（工具链升级这一格比对看不见），强制跑一趟"
fi

if git -C "$ROOT" diff --quiet "$green" "$staged_tree" -- "${input_paths[@]}"; then
  say_skip "与上次整轮全绿那棵树（${green:0:12}）在这几条路径上逐字相同：${inputs} ${INPUT_TABLE}"
fi
changed="$(git -C "$ROOT" diff --name-only "$green" "$staged_tree" -- "${input_paths[@]}" | head -5 | tr '\n' ' ')"
say_run "这几条路径与上次整轮全绿那棵树不同：${changed}"
