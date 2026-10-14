#!/usr/bin/env bash
# Y1-丙 喂法：叫停轮登记行的出处写「records/log.md 第 3 行」，登记时那一行有轮名与用户原话；之后照常在 records/log.md 前面插一行（加小节标题），
# 同一张表、同一个出处，判法库（⑥ 与门禁 abandoned-rounds 格同读的 lib_abandoned_rounds.py）前后各判一次。
# 用法：y1_drift.sh <判法所在副本> <树的根目录（新）>   输出：插行前、插行后库的判定
set -euo pipefail
judge=$(cd "$1" && pwd); root=$2
mkdir -p "$root/records" "$root/research/prompts/dr-r1-snapshot"
printf '# 样本调度\n\n- 2026-10-13 用户叫停 dr-r1：「这一轮先放一放，别再派腿了」\n- dr-r1 之后的活并进别处。\n' > "$root/records/log.md"
printf '# body\n' > "$root/research/prompts/_dr-r1-body.md"; printf '# out\n' > "$root/research/prompts/dr-r1-opus-output.md"
printf '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  crates/foo/src/lib.rs\n' > "$root/research/prompts/dr-r1-snapshot/frozen-sha256.txt"
printf 'dr-r1\t2026-10-13\t用户叫停；出处 records/log.md 第 3 行\t并进别处\n' > "$root/research/prompts/abandoned-rounds.tsv"
echo "--- 插行前 records/log.md 第 3 行：$(sed -n 3p "$root/records/log.md")"
PYTHONDONTWRITEBYTECODE=1 python3 -B -I "$judge/.claude/hooks/lib_abandoned_rounds.py" "$root" 2>&1 | sed "s#$root#<TREE>#g"
sed -i '2a ## 批 9（2026-10-14）\n' "$root/records/log.md"
echo "--- 插行后 records/log.md 第 3 行：$(sed -n 3p "$root/records/log.md")"
PYTHONDONTWRITEBYTECODE=1 python3 -B -I "$judge/.claude/hooks/lib_abandoned_rounds.py" "$root" 2>&1 | sed "s#$root#<TREE>#g"
