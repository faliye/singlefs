#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：由它调的 research/scripts/replay.sh 编实验二进制时用，缺了在那一步判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）
# gate-stage: 入库的实验数今天还复现得出来吗
#
# 判据：把纯模型实验重跑一遍，和 `research/results/` 里那份原始产物**逐字节**比对。
# 对不上不等于代码坏了，也可能是产物该更新了——两种都要人来判，所以判红。
#
# ⚠️ **默认跑表里所有 exact 行，减掉慢的那几个。** 慢的那几个（下面的 SLOW 清单里判定为 exact 的行）
# 与**表里判定为 timing 的每一行**都压在 `GATE_REPLAY_FULL=1` 后面，
# 且**本阶段会把它们逐个列成「本次没跑」**——
# `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」。
# 样本：fixtures/87-replay.sh/green 是一个带假 replay.sh 的小仓，表里有一行 exact 的快实验、一行 SLOW 里的 exact、一行 SLOW 里只有 timing 行的，
# 快的那行要派给 replay.sh，另外两行都要逐个列进「没派」；red 的表一行都没有，必须判红。
#
# ⚠️ **没跑的清单要从表里现算，不许只列 SLOW。** 2026-09-16 现查：默认档跑 119 个 exact 行，
# 而表里 9 个 timing 行一个都不跑，报告却只列了 E9 / E17 / E20 / E21——
# **E44 / E58 / E107 / E128 / E140 / E144 六个既没跑，也没出现在任何一行里**。
# 当轮把 `GATE_REPLAY_FULL=1` 补跑了一次才发现：E58 的产物早已对不上（源码的指针载荷
# 108 → 53 改过而产物没重跑），E17 在 release 下直接 panic 跑不起来——而默认档一直是绿的。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"

# 这次改动没碰这道阶段判的东西就退 77（本次未跑），不退 0（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。
# 它判的是入库实验数今天还复现不复现得出来，输入是实验二进制、replay.sh 的表与留存产物；
# research/prompts/ 是冻结证据，改它不可能改变任何实验的输出，所以不在前缀里。C8（范围判定）的粗粒度前身。
# 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
  "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "这次改动没碰它判的东西" \
  "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 research/scripts/change-touches-crates.sh。" \
  -- "$ROOT" crates/ research/e7-index-bench/ research/scripts/ research/results/
# ⚠️ **不许写死清单。** 第一版把 FAST 手抄成 22 个编号，此后新增的实验
# （E40 起共 14 个）虽然进了 replay.sh 的表，却**一个都没被门禁跑过**，
# 而本阶段仍旧打印「本次没跑：E9 E17 E20 E21」——**列出来的那句话本身是假的**。
# 这正是 `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」要拦的形态：
# 漏跑不可怕，漏跑而且报告说跑了才可怕。
# ⇒ 改成从 replay.sh 的表里现算：取判定为 exact 的行，减掉慢的那几个。
SLOW=(E9 E17 E20 E21)
mapfile -t ALL_EXACT < <(sed -n 's/^\(E[0-9]*\)|[^|]*|[^|]*|[^|]*|exact$/\1/p' research/scripts/replay.sh)
if ((${#ALL_EXACT[@]} == 0)); then
  echo "  ✗ 从 research/scripts/replay.sh 里一行都没读到——表的格式变了？"
  echo "    → 本阶段靠 '编号|二进制|参数|产物|exact' 这个形状取行，改表格式要同时改这里。"
  exit 1
fi
mapfile -t ALL_TIMING < <(sed -n 's/^\(E[0-9]*\)|[^|]*|[^|]*|[^|]*|timing$/\1/p' research/scripts/replay.sh)
FAST=(); SKIPPED=()
for e in "${ALL_EXACT[@]}"; do
  skip=0
  for s in "${SLOW[@]}"; do [[ "$e" == "$s" ]] && skip=1; done
  if ((skip)); then SKIPPED+=("$e（慢）"); else FAST+=("$e"); fi
done
# timing 行默认一行都不跑：逐个进「没跑」清单。去重只比已经进了清单的那几行（同一个编号既有 exact 行又有 timing 行时），
# 不比 SLOW：SLOW 里的编号若只有 timing 行，拿 SLOW 去重会让它既不跑、也不进清单。
for e in "${ALL_TIMING[@]}"; do
  dup=0
  for s in ${SKIPPED[@]+"${SKIPPED[@]}"}; do [[ "$s" == "$e（慢）" || "$s" == "$e（计时）" ]] && dup=1; done
  ((dup)) || SKIPPED+=("$e（计时）")
done

if [[ "${GATE_REPLAY_FULL:-0}" == 1 ]]; then
  bash research/scripts/replay.sh || exit 1
  exit 0
fi

bash research/scripts/replay.sh "${FAST[@]}" || exit 1
# 派给 replay.sh 的行里，登记了准入（.claude/gate.d/stage-inputs.tsv）而输入自留存产物以来没变的不跑，
# replay.sh 汇总行报「输入没变没跑 N」并逐个列名；这里不再说成「跑了」。
echo "  ! 本次派给 replay.sh ${#FAST[@]} 个（表里的 exact 行减掉慢的；其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；没派 ${#SKIPPED[@]} 个：${SKIPPED[*]}"
echo "    标（慢）的是 SLOW 清单（${SLOW[*]}）里判定为 exact 的行；标（计时）的是表里判定为 timing 的行，默认档一行都不跑。"
echo "    跑： GATE_REPLAY_FULL=1 bash .claude/scripts/gate.sh"
echo "    ⚠️ 它们的结论靠 replay.sh 里的区间断言钉住，不跑就等于那几条断言本轮没验。"
