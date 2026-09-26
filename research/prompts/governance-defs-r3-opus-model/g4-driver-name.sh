#!/usr/bin/env bash
# G4：执行员第 5 步「入库装置照 E156、E158 的先例另写一个 `driver_e<号>` 驱动函数，登记行写 E<号>|@driver_e<号>||<产物>|exact」
# 照字面给同一个实验的第二份产物再写一个 driver_e<号>：bash 里后定义的函数盖掉先定义的，第一行登记跑的是第二个函数。
# 调用方式照 research/scripts/replay.sh 的 replay_one（`"${bin#@}" >"$fresh"`）。
# 用法：bash g4-driver-name.sh <仓根>
set -uo pipefail
real="$1"
echo "今天 replay.sh 里 @driver 登记行按实验号数："
grep -oE '^E[0-9]+\|@driver_[a-z0-9_]+' "$real/research/scripts/replay.sh" | cut -d'|' -f1 | sort | uniq -c | sort -rn
echo "今天 replay.sh 里名字不是恰好 driver_e<号> 的驱动函数：$(grep -oE '^driver_e[0-9]+_[a-z0-9_]+\(\)' "$real/research/scripts/replay.sh" | wc -l) 个（例：$(grep -oE '^driver_e[0-9]+_[a-z0-9_]+\(\)' "$real/research/scripts/replay.sh" | head -2 | tr '\n' ' ')）"
echo "今天 .claude/gate.d/ 与 .claude/hooks/ 里提到 driver_ 的文件：$(grep -l 'driver_' "$real"/.claude/gate.d/*.sh "$real"/.claude/hooks/*.sh 2>/dev/null | wc -l) 个；replay.sh 里查函数重名（declare -F / type -t）的行：$(grep -cE 'declare -F|type -t' "$real/research/scripts/replay.sh")"
table='E900|@driver_e900||e900-first.out|exact
E900|@driver_e900||e900-second-mode.out|exact'
driver_e900() { echo "first-mode output"; }
# 第二次照模板追加（重跑已有入库装置、换一个模式出第二份产物）
driver_e900() { echo "second-mode output"; }
while IFS='|' read -r exp bin _ stored _; do
  printf '%s %s 期望比 %s，实际跑出：%s\n' "$exp" "$bin" "$stored" "$("${bin#@}")"
done <<<"$table"
