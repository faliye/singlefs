#!/usr/bin/env bash
# gate-stage: 入库的实验数今天还复现得出来吗
#
# 判据：把纯模型实验重跑一遍，和 `research/results/` 里那份原始产物**逐字节**比对。
# 对不上不等于代码坏了，也可能是产物该更新了——两种都要人来判，所以判红。
#
# ⚠️ **默认只跑得快的那 19 个（约 45 秒）。** 计时实验（E9 的 25 段 O_DIRECT、
# E17、E20、E21）要几分钟，压在 `GATE_REPLAY_FULL=1` 后面，
# 且**本阶段会把它们显式列成「本次没跑」**——
# `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」。
set -uo pipefail
FAST=(E14 E18 E19 E24 E25 E26 E27 E28 E29 E30 E31 E32 E33 E34 E36 E37 E38 E8 E16)
SLOW=(E9 E17 E20 E21)

if [[ "${GATE_REPLAY_FULL:-0}" == 1 ]]; then
  bash research/scripts/replay.sh || exit 1
  exit 0
fi

bash research/scripts/replay.sh "${FAST[@]}" || exit 1
echo "  ! 本次没跑：${SLOW[*]}（E9 是 25 段 O_DIRECT，其余三个是计时实验，合计几分钟）"
echo "    跑： GATE_REPLAY_FULL=1 bash .claude/scripts/gate.sh"
echo "    ⚠️ 它们的结论靠 replay.sh 里的区间断言钉住，不跑就等于那几条断言本轮没验。"
