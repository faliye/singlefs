#!/usr/bin/env bash
# Z7 喂法：长活声明豁免表的只缩不涨（lib_long_run.py exempt-check；门禁 long-run-header-exempt 格调同一份，基准里有表才传 --base-table）。
#   repin        改被钉的入口脚本正文一处并重钉表里的 sha256：基准里有这张表（--base-table 冻结副本那一版）与基准里还没有这张表（不传）各判一次
#   reuse-entry  不碰被钉的入口：改它 exec 的编排脚本（没被钉），另放一份下一次修订的配置，用同一个入口起；
#                判 exempt-check（带基准）与 lib_long_run.py script（detached-run.sh start / long-run-check 判被起脚本的同一份）
# 树用硬链接拷（cp -al），被改的文件先换成新 inode；冻结副本不被碰。用法：z7_exempt_repin.sh <名>=<冻结副本根> <工作目录>
set -euo pipefail
trim() { python3 -c 'import sys; n = int(sys.argv[1]); sys.stdout.write("".join(line.rstrip("\n")[:n] + "\n" for line in sys.stdin))' "$1"; }   # 按字截（cut -c 按字节截，会把中文切坏）
label="${1%%=*}" frozen="${1#*=}" work="$2"
research=scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes
entry="$research/run_segment_one_revision_11_multi_host.sh"
orchestrator="$research/segment_one_units_revision_11.py"
table=.claude/long-run-live-hosts-exempt
tree="$work/$label-tree"
mkdir -p "$work"; rm -rf "$tree"; cp -al "$frozen" "$tree"
for path in "$table" "$entry" "$orchestrator"; do cp --remove-destination "$frozen/$path" "$tree/$path"; done
lib="$tree/.claude/hooks/lib_long_run.py"
check() {   # 名 [--base-table 文件]
  local name="$1"; shift
  out="$(PYTHONDONTWRITEBYTECODE=1 python3 -B "$lib" exempt-check "$tree" "$@" 2>&1)" && code=0 || code=$?
  printf '%s\t%s\texempt-check 退出=%s\t%s\n' "$label" "$name" "$code" "$(tr '\n' ' ' <<<"$out" | trim 300)"
}
check 未改
# repin：入口正文改一处（nice 的优先级），表里那一行的 sha256 换成新文件的
python3 -B - "$tree/$entry" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = 'exec nice -n 19 python3 -B "$RESEARCH/segment_one_units_revision_11.py" run'
assert text.count(old) == 1
open(path, "w", encoding="utf-8").write(text.replace(old, 'exec nice -n 10 python3 -B "$RESEARCH/segment_one_units_revision_11.py" run'))
PY
old_sha="$(sha256sum "$frozen/$entry" | cut -d' ' -f1)" new_sha="$(sha256sum "$tree/$entry" | cut -d' ' -f1)"
python3 -B - "$tree/$table" "$old_sha" "$new_sha" <<'PY'
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
assert text.count(old) == 1
open(path, "w", encoding="utf-8").write(text.replace(old, new))
PY
check repin-基准有表 --base-table "$frozen/$table"
check repin-基准没有表
# reuse-entry：入口与表还原成冻结副本那一版，只改没被钉的编排脚本并放一份新配置
cp --remove-destination "$frozen/$entry" "$tree/$entry"; cp --remove-destination "$frozen/$table" "$tree/$table"
printf '\n# 下一次修订（样本）：单元表换成新的一批，入口照旧用修订 11 那一份，配置给新的一份\n' >> "$tree/$orchestrator"
cp "$tree/$research/revision_11_placeholders.json" "$tree/$research/revision_13_placeholders.json"
check reuse-entry-基准有表 --base-table "$frozen/$table"
out="$(cd "$tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B "$lib" script "$tree" -- bash "$entry" "$research/revision_13_placeholders.json" 2>&1)" && code=0 || code=$?
printf '%s\treuse-entry-被起脚本\tscript 退出=%s\t%s\n' "$label" "$code" "$(tr '\n' ' ' <<<"$out" | trim 300)"
rm -rf "$tree"
