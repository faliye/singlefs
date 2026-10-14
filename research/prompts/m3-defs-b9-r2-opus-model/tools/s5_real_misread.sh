#!/usr/bin/env bash
# S5 真树喂法：真记录 records/2026-10-01-删除与并发事务.md 第 1594 行与第 1595 行都带 m3-batch63-code-r1 与「」用户原话；
# 第 1594 行说的是「判决写完走提交前那几步」（这一轮接着收尾、写判决），第 1595 行才是两条腿停掉、不写判决。
# 历史：用户说完第 1594 行那句、还没说第 1595 行那句时，主 agent 把「不要再打了」读成叫停，登记一行，〔〕里写的是转述（不是逐字照抄），行号写 1594。
# 树用硬链接拷，记录截到第 1594 行（那一刻的样子）、登记表那一行换掉，两份都先换新 inode。判两样：叫停表判法与派发闸 ⑥（派实现员改那一轮快照里的文件）。
# 用法：s5_real_misread.sh <名>=<冻结副本根> <工作目录>
set -euo pipefail
trim() { python3 -c 'import sys; n = int(sys.argv[1]); sys.stdout.write("".join(line.rstrip("\n")[:n] + "\n" for line in sys.stdin))' "$1"; }
label="${1%%=*}" frozen="${1#*=}" work="$2"
record="records/2026-10-01-删除与并发事务.md" table="research/prompts/abandoned-rounds.tsv"
tree="$work/$label-tree"
mkdir -p "$work"; rm -rf "$tree"; cp -al "$frozen" "$tree"
rm -f "$tree/$record" "$tree/$table"
head -n 1594 "$frozen/$record" > "$tree/$record"
grep -v '^m3-batch63-code-r1	' "$frozen/$table" > "$tree/$table"
printf 'm3-batch63-code-r1\t2026-10-11\t用户 2026-10-11 叫停（records/2026-10-01-删除与并发事务.md 第 1594 行〔m3-batch63-code-r1 用户叫停，不再打〕）\t不写判决\n' >> "$tree/$table"
prompt="$work/$label-writer-prompt.txt"
printf '%s\n' '开工先读 `.claude/agent-common.md`。' '重型测试：不跑' "草稿目录：$work/$label-scratch/" "报告：$work/$label-scratch/report.md" \
  '条款：照第 63 批的结论给判定库的一处断言补一行注释。' '要动的 crates 文件：crates/singlefs-checker-tier/src/verdict_store.rs' '预计挂钟：2 小时内' '线程上限：31' > "$prompt"
lib="$(cd "$tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B .claude/hooks/lib_abandoned_rounds.py . 2>&1 | grep m3-batch63-code-r1 | trim 200)" || true
dispatch="$(cd "$tree" && CLAUDE_PROJECT_DIR="$tree" PYTHONDONTWRITEBYTECODE=1 bash .claude/hooks/runner-dispatch-guard.sh --check-prompt "$prompt" --type implementation-writer --root "$tree" 2>&1)" && code=0 || code=$?
printf '%s\t叫停表：%s\t⑥ 派实现员退出=%s\t%s\n' "$label" "$lib" "$code" "$(grep -E '^✗|三方轮 m3-batch63|放行' <<<"$dispatch" | head -2 | tr '\n' ' ' | trim 200)"
rm -rf "$tree"
