#!/usr/bin/env bash
# gate-stage: 入库的实验数今天还复现得出来吗
#
# 判据：把纯模型实验重跑一遍，和 `research/results/` 里那份原始产物**逐字节**比对。
# 对不上不等于代码坏了，也可能是产物该更新了——两种都要人来判，所以判红。
#
# ⚠️ **默认跑表里所有 exact 行，减掉慢的那几个。** 计时实验（E9 的 25 段 O_DIRECT、
# E17、E20、E21）要几分钟，压在 `GATE_REPLAY_FULL=1` 后面，
# 且**本阶段会把它们显式列成「本次没跑」**——
# `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」。
set -uo pipefail
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
FAST=()
for e in "${ALL_EXACT[@]}"; do
  skip=0
  for s in "${SLOW[@]}"; do [[ "$e" == "$s" ]] && skip=1; done
  ((skip)) || FAST+=("$e")
done

if [[ "${GATE_REPLAY_FULL:-0}" == 1 ]]; then
  bash research/scripts/replay.sh || exit 1
  exit 0
fi

bash research/scripts/replay.sh "${FAST[@]}" || exit 1
echo "  ! 本次跑了 ${#FAST[@]} 个；没跑：${SLOW[*]}（E9 是 25 段 O_DIRECT，其余三个是计时实验，合计几分钟）"
echo "    跑： GATE_REPLAY_FULL=1 bash .claude/scripts/gate.sh"
echo "    ⚠️ 它们的结论靠 replay.sh 里的区间断言钉住，不跑就等于那几条断言本轮没验。"
