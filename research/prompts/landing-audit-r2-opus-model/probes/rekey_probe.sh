#!/usr/bin/env bash
# replay-comparison-units 格「复跑写 none: 的行只缩不涨」：键是（登记出处，留存产物名），删掉一行 none 行、换成另一个实验的比对对象借用同一对键，照绿。
# 格的 python 从仓里那一份 doc-experiments.sh 现取（cell_replay_comparison_units 里 <<'PY' 到 PY 那一段），在一个造出来的小仓里起。
# 用法：bash rekey_probe.sh <仓根> <新的空目录>
set -uo pipefail
repo="$1" work="$2"
mkdir -p "$work/repo"
awk '/^cell_replay_comparison_units\(\) \{/{f=1} f&&/<<.PY.$/{p=1;next} p&&/^PY$/{exit} p{print}' "$repo/.claude/gate.d/doc-experiments.sh" > "$work/cell.py"
cd "$work/repo" || exit 1
git init -q && git config user.email probe@example.invalid && git config user.name probe
mkdir -p .claude/gate.d research/prompts src scripts/experiment-apparatus
printf 'x\n' > research/prompts/e165-r5-prereg.md; printf 'x\n' > research/prompts/e200-r1-prereg.md
printf 'const U: [&str; 2] = ["r5 合法", "甲"];\n' > src/e165.rs; printf 'const U: [&str; 1] = ["乙新对象"];\n' > src/e200.rs
printf '#!/usr/bin/env bash\nE165|@driver_e165_a||e165-a.out|exact\n' > scripts/experiment-apparatus/replay.sh
units=.claude/gate.d/replay-comparison-units.tsv
driver_row=$'E165\tresearch/prompts/e165-r5-prereg.md\t甲\tsrc/e165.rs\te165-a.out\tdriver_e165_a'
printf '# 表头\n%s\nE165\tresearch/prompts/e165-r5-prereg.md\tr5 合法\tsrc/e165.rs\tsegment3-r5-legal.out\tnone:第三段新跑的，被比的一侧\n' "$driver_row" > "$units"
git add -A && git commit -qm 基准
judge() { echo "name=$1"; python3 -I "$work/cell.py" "$units" scripts/experiment-apparatus/replay.sh HEAD "" "$repo/.claude/gate.d/lib-lag-baseline.py"; echo "exit=$?"; }
judge 基准原样
printf '# 表头\n%s\nE200\tresearch/prompts/e165-r5-prereg.md\t乙新对象\tsrc/e200.rs\tsegment3-r5-legal.out\tnone:没有复跑行的新比对对象\n' "$driver_row" > "$units"
judge 删掉E165的none行换成E200的新比对对象借用同一对键
printf '# 表头\n%s\nE200\tresearch/prompts/e200-r1-prereg.md\t乙新对象\tsrc/e200.rs\tsegment3-r5-legal.out\tnone:没有复跑行的新比对对象\n' "$driver_row" > "$units"
judge 对照登记出处写成新实验自己的登记
