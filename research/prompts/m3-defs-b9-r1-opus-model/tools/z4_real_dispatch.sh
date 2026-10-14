#!/usr/bin/env bash
# Z4 真树喂法：这一版冻结副本里已登记、已收的叫停轮 m3-batch63-code-r1，出处那份调度记录之后追加一行补记、抄了登记用的那段原话。
# 判两样：叫停表的判法（lib_abandoned_rounds.py，门禁 abandoned-rounds 格调同一份）；派发闸 ⑥（--check-prompt 派实现员改那一轮快照里的 crates/singlefs-checker-tier/src/verdict_store.rs）。
# 树用硬链接拷（cp -al），被改的那一份记录先换成新 inode 再追加，冻结副本不被碰。
# 用法：z4_real_dispatch.sh <名>=<冻结副本根> <工作目录>
set -euo pipefail
trim() { python3 -c 'import sys; n = int(sys.argv[1]); sys.stdout.write("".join(line.rstrip("\n")[:n] + "\n" for line in sys.stdin))' "$1"; }   # 按字截（cut -c 按字节截，会把中文切坏）
label="${1%%=*}" frozen="${1#*=}" work="$2"
record="records/2026-10-01-删除与并发事务.md"
excerpt='去向：m3-batch63-code-r1 两条腿当场停掉（没交回、不写判决）'
tree="$work/$label-tree"
mkdir -p "$work"; rm -rf "$tree"; cp -al "$frozen" "$tree"
cp --remove-destination "$frozen/$record" "$tree/$record"     # 断开硬链接：之后追加只改拷贝
prompt="$work/$label-writer-prompt.txt"
printf '%s\n' '开工先读 `.claude/agent-common.md`。' '重型测试：不跑' "草稿目录：$work/$label-scratch/" "报告：$work/$label-scratch/report.md" \
  '条款：照第 63 批的结论给判定库的一处断言补一行注释。' '要动的 crates 文件：crates/singlefs-checker-tier/src/verdict_store.rs' '预计挂钟：2 小时内' '线程上限：31' > "$prompt"
state() {
  local lib dispatch
  lib="$(cd "$tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B .claude/hooks/lib_abandoned_rounds.py . 2>&1 | grep m3-batch63-code-r1 | trim 220)" || true
  dispatch="$(cd "$tree" && CLAUDE_PROJECT_DIR="$tree" PYTHONDONTWRITEBYTECODE=1 bash .claude/hooks/runner-dispatch-guard.sh --check-prompt "$prompt" --type implementation-writer --root "$tree" 2>&1)" && code=0 || code=$?
  printf '%s\t%s\t叫停表：%s\t⑥ 派实现员退出=%s\t%s\n' "$label" "$1" "$lib" "$code" "$(grep -E '^✗|三方轮 m3-batch63' <<<"$dispatch" | head -2 | tr '\n' ' ' | trim 260)"
}
state 追加之前
printf '%s\n' "- 补记：m3-batch63-code-r1 的叫停登记已收进 research/prompts/abandoned-rounds.tsv，出处抄的是第 1595 行「$excerpt」。" >> "$tree/$record"
state 追加之后
rm -rf "$tree"
