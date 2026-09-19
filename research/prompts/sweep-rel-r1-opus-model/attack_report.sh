#!/usr/bin/env bash
# sweep-rel-r1 攻方腿（Opus）：A3 —— --check-report 套话过、--groups 漏组多组。只读仓，产物写进 $1（草稿目录）。
# 用法：cd 仓根 && bash research/prompts/sweep-rel-r1-opus-model/attack_report.sh 草稿目录
set -u
D=$1; T=research/scripts/stale-candidates.py; F=research/scripts/fixtures/stale-candidates-benchmark-facts.tsv
rm -f "$D/cand.tsv"
python3 $T --facts $F --base b1c8cef~1 --target 00c9d4f --out "$D/cand.tsv" | tail -1
# 套话报告：候选表逐行机械生成，不读任何一行的上下文
gen() { # $1 判定 $2 理由 $3 输出
  { echo '## 逐行判定'; echo '| 组 | 载体 | 判定 | 改后的句子或理由 |'; echo '|---|---|---|---|'
    tail -n +2 "$D/cand.tsv" | awk -F'\t' -v v="$1" -v r="$2" '{print "| " $1 " | " $4 " | " v " | " r " |"}'; } > "$3"; }
gen '不相干' '这一行说的不是这件事实的现状' "$D/R1.md"
gen '要人看' '这一行要主 agent 对着结束那一版再看' "$D/R2.md"
gen '要改' '........' "$D/R3.md"
gen '事件句不改nonsense' 'xxxxxxxx' "$D/R4.md"
for r in R1 R2 R3 R4; do printf '%s: ' $r; python3 $T --check-report "$D/cand.tsv" "$D/$r.md" | tail -1; echo "  exit=${PIPESTATUS[0]}"; done
# 真判法也可能被误红：「要改」给的改后句子本身是一行 kb 表格（带 |）
{ echo '## 逐行判定'; echo '| 组 | 载体 | 判定 | 改后的句子或理由 |'; echo '|---|---|---|---|'
  tail -n +2 "$D/cand.tsv" | head -1 | awk -F'\t' '{print "| " $1 " | " $4 " | 要改 | \\| C22（示例） \\| 前置：层 0 两条流，含释放与重用 \\| |"}'
  tail -n +3 "$D/cand.tsv" | awk -F'\t' '{print "| " $1 " | " $4 " | 不相干 | 这一行说的不是这件事实的现状 |"}'; } > "$D/R5.md"
printf 'R5（第一行改后句子是转义过的表格行）: '; python3 $T --check-report "$D/cand.tsv" "$D/R5.md" | tail -2 | head -1; echo "  exit=${PIPESTATUS[0]}"
echo '--- --groups 解析（候选表里实际有的组号）'
tail -n +2 "$D/cand.tsv" | cut -f1 | sort -u | tr '\n' ' '; echo
: > "$D/empty.md"
for g in 'F6-F1' 'Z1' 'F1-G3' 'F1-F12' 'F7b' 'F01' 'A1-A9,B1-B16,C1-C9,D1-D14,E1,F1-F12,G1-G4,H1-H4'; do
  printf -- '--groups %-60s ' "$g"; python3 $T --check-report "$D/cand.tsv" "$D/empty.md" --groups "$g" | tail -1 | cut -c1-80; echo "  exit=${PIPESTATUS[0]}"
done
printf 'F1-F12 选中的候选行数 vs F7b 的候选行数：'
python3 - "$D/cand.tsv" <<'PY'
import sys, importlib.util
spec = importlib.util.spec_from_file_location('sc', 'research/scripts/stale-candidates.py'); sc = importlib.util.module_from_spec(spec); spec.loader.exec_module(sc)
keys = sc.read_candidate_keys(sys.argv[1]); sel = sc.parse_group_ranges('F1-F12')
print(sum(1 for k in keys if k[0] in sel), sum(1 for k in keys if k[0] == 'F7b'), '；F1-G3 →', sorted(sc.parse_group_ranges('F1-G3')), '；F6-F1 →', sc.parse_group_ranges('F6-F1'))
PY
