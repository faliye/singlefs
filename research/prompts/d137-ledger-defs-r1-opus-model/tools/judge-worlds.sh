#!/usr/bin/env bash
# 在每个样本根上单跑门禁 doc-experiments 的 preregistration-judge-features 格，逐个打退出码；门禁输出写进 <样本目录>/<世界名>.gate.out。
# 用法（在仓根起）：bash <模型目录>/tools/judge-worlds.sh <make-worlds.sh 造出的目录>
# 期望与「照定义该判成什么」在 <模型目录>/out/expected.txt。退出码：0 七个都跑了（不判对错）；2 不在仓根或缺参数。
set -uo pipefail
worlds="${1:?要给 make-worlds.sh 造出的目录}"
[[ -f .claude/gate.d/doc-experiments.sh ]] || { echo "  ✗ 要在仓根起（找不到 .claude/gate.d/doc-experiments.sh）" >&2; exit 2; }
for name in control-single-line-red control-single-line-green continued-bin-on-next-line continued-features-on-next-line marker-without-judge alias-import-judge declared-cargo-test-green; do
  SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 preregistration-judge-features 格; 范围=1 格; 为什么=在攻方腿造的样本根上核续行、别名导入与只数不判的装置这一格怎么判' bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features "$worlds/$name" > "$worlds/$name.gate.out" 2>&1
  rc=$?
  echo "name=judge_world world=$name rc=$rc last=$(grep -E '✓|✗|!' "$worlds/$name.gate.out" | grep -v '汇总' | tail -n 1 | cut -c1-160)"
done
