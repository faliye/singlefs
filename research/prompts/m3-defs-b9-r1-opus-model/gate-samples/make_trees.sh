#!/usr/bin/env bash
# 给主 agent 跑门禁格用的小树（子 agent 起不了 --check）。每棵都从冻结副本硬链接拷（cp -al），被改的文件先换成新 inode；冻结副本不被碰。
# 用法：make_trees.sh <冻结副本根> <模型目录> <新的空目录 T>
#   T/abandoned-after   叫停表收的 m3-batch63-code-r1：出处那份调度记录末尾追加一行补记，抄了登记用的那段原话（不是 git 仓，abandoned-rounds 格只看树）
#   T/htd-insert        random_histories.rs 里新加一个 available_parallelism 默认线程数函数（排除表那一行锚串 available_parallelism）
#   T/htd-edit          crates-mutation-rows.py 排除表钉住的那一行改几个字，读进 GATE_MUTATION_TOTAL_WORKERS
#   T/htd-shell         模型目录 feeds/z2/sample 原样拷（单独一棵，含 capped.sh、登记与排除表五行）
#   T/z5                模型目录 feeds/z5/setup.sh 建的 git 样本（假配置，值是编的）
#   T/exempt-base-table 与 T/exempt-no-base：git 仓；前者基准提交里有豁免表与被钉的入口、工作区改入口并重钉；后者基准提交里没有豁免表（表与改动都在工作区）
set -euo pipefail
frozen="$(cd "$1" && pwd)" model="$(cd "$2" && pwd)" T="$3"
mkdir -p "$T"
record="records/2026-10-01-删除与并发事务.md"
cp -al "$frozen" "$T/abandoned-after"
cp --remove-destination "$frozen/$record" "$T/abandoned-after/$record"
printf '%s\n' '- 补记：m3-batch63-code-r1 的叫停登记已收进 research/prompts/abandoned-rounds.tsv，出处抄的是第 1595 行「去向：m3-batch63-code-r1 两条腿当场停掉（没交回、不写判决）」。' >> "$T/abandoned-after/$record"
cp -al "$frozen" "$T/htd-insert"
f=crates/singlefs-harness/tests/random_histories.rs
cp --remove-destination "$frozen/$f" "$T/htd-insert/$f"
python3 -B - "$T/htd-insert/$f" <<'PY'
import sys
p = sys.argv[1]; t = open(p, encoding="utf-8").read(); old = "fn worker_threads_by_default() -> usize {\n"
assert t.count(old) == 1
open(p, "w", encoding="utf-8").write(t.replace(old, "fn sampling_workers_by_default() -> usize {\n    std::thread::available_parallelism().map_or(1, usize::from)\n}\n\n" + old))
PY
cp -al "$frozen" "$T/htd-edit"
f=scripts/mutation/crates-mutation-rows.py
cp --remove-destination "$frozen/$f" "$T/htd-edit/$f"
python3 -B - "$T/htd-edit/$f" <<'PY'
import sys
p = sys.argv[1]; t = open(p, encoding="utf-8").read(); old = "        processor_count = os.cpu_count() or 4"
assert t.count(old) == 1
open(p, "w", encoding="utf-8").write(t.replace(old, '        processor_count = int(os.environ.get("GATE_MUTATION_TOTAL_WORKERS") or 0) or os.cpu_count() or 4'))
PY
cp -a "$model/feeds/z2/sample" "$T/htd-shell"
mkdir -p "$T/z5"; ( cd "$T/z5" && bash "$model/feeds/z5/setup.sh" )
research=scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes
entry="$research/run_segment_one_revision_11_multi_host.sh"
for name in exempt-base-table exempt-no-base; do
  t="$T/$name"; mkdir -p "$t/.claude/hooks" "$t/$research"
  cp "$frozen/.claude/hooks/lib_long_run.py" "$frozen/.claude/hooks/lib_shell_words.py" "$t/.claude/hooks/"
  cp "$frozen/$entry" "$t/$entry"
  grep -F "$entry" "$frozen/.claude/long-run-live-hosts-exempt" > "$t/.claude/long-run-live-hosts-exempt"
  ( cd "$t" && git init -q . && git config user.email t@example.com && git config user.name t
    if [[ "$name" == exempt-base-table ]]; then git add -A; else git add .claude/hooks "$entry"; fi
    git commit -qm base
    sed -i 's/exec nice -n 19 python3 -B "$RESEARCH\/segment_one_units_revision_11.py" run/exec nice -n 10 python3 -B "$RESEARCH\/segment_one_units_revision_11.py" run/' "$entry"
    new="$(sha256sum "$entry" | cut -d' ' -f1)"; old="$(sha256sum "$frozen/$entry" | cut -d' ' -f1)"
    sed -i "s/$old/$new/" .claude/long-run-live-hosts-exempt )
done
echo "建好：$(ls "$T" | tr '\n' ' ')"
