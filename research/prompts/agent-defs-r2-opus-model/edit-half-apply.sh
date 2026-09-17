#!/usr/bin/env bash
# agent-defs-r2 云端攻方腿模型一：kb-scribe 第 1 步「replace-batch.py --dry-run 核完、实写用 Edit 逐条改」
# 在「核完之后、逐条写的中途」有别的写入（另一个书记员的 relabel-item.py）或被打断时，留下半套改动；
# 对照：第一轮之前的做法（replace-batch.py 两阶段实写）同一段历史下一处都不写。
# Edit 用 research/scripts/replace-once.py 模拟：共用约束 .claude/agent-common.md 第 15 行写「旧串必须在文件里恰好命中一次，与 replace-once.py 同义」。
# 只在临时目录里造文件，不碰仓里任何文件；仓里的两个脚本只读调用。
#   TMPDIR=<草稿目录> bash edit-half-apply.sh
set -uo pipefail
REPO="${REPO:-/home/fy5090/code/singlefs}"
BATCH="$REPO/research/scripts/replace-batch.py"
ONCE="$REPO/research/scripts/replace-once.py"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK:?}"' EXIT

make_file() {
  printf '%s\n' \
    '| 5 | 甲字段 | **已定**：旧说法。 |' \
    '- 这一格的宽度同 D22（单元原子性怎么合成） 已定项 7 的根记录字段表。' \
    '| 9 | 乙字段 | **已定**：旧说法。 |' > "$1"
}
make_spec() {
  python3 - "$1" "$2" <<'PY'
import json, sys
path = sys.argv[2]
json.dump([
  {"file": path, "old": "| 5 | 甲字段 | **已定**：旧说法。 |", "new": "| 5 | 甲字段 | **已定**：新说法。 |"},
  {"file": path, "old": "- 这一格的宽度同 D22（单元原子性怎么合成） 已定项 7 的根记录字段表。", "new": "- 这一格的宽度同 D22（单元原子性怎么合成） 已定项 7 的根记录字段表（新注）。"},
  {"file": path, "old": "| 9 | 乙字段 | **已定**：旧说法。 |", "new": "| 9 | 乙字段 | **已定**：新说法。 |"},
], open(sys.argv[1], "w", encoding="utf-8"), ensure_ascii=False)
PY
}
# 别的书记员第 3 步的 relabel-item.py 对 D22 第 7 项翻回未定时，对这一行做的改写（与 relabel 的输出形态同：已定项 7 → 未定项 7）
foreign_relabel() { python3 -c 'import sys;p=sys.argv[1];t=open(p,encoding="utf-8").read();open(p,"w",encoding="utf-8").write(t.replace("D22（单元原子性怎么合成） 已定项 7","D22（单元原子性怎么合成） 未定项 7"))' "$1"; }
applied() { grep -c '新说法\|（新注）' "$1"; }

run_case() {  # $1 名字  $2 写法 each|batch  $3 干扰 none|relabel_after_first|interrupt_after_first
  local name="$1" mode="$2" event="$3" dir file spec rc_dry rc_last=0 k
  dir="$WORK/$name"; mkdir -p "$dir"; file="$dir/decision.md"; spec="$dir/spec.json"
  make_file "$file"; make_spec "$spec" "$file"
  python3 "$BATCH" --dry-run "$spec" >/dev/null; rc_dry=$?
  if [[ "$mode" == batch ]]; then
    [[ "$event" == relabel_after_first ]] && foreign_relabel "$file"
    python3 "$BATCH" "$spec" >/dev/null; rc_last=$?
  else
    for k in 0 1 2; do
      if [[ $k -eq 1 && "$event" == relabel_after_first ]]; then foreign_relabel "$file"; fi
      if [[ $k -eq 1 && "$event" == interrupt_after_first ]]; then rc_last=interrupted; break; fi
      old="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))[int(sys.argv[2])]["old"])' "$spec" $k)"
      new="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))[int(sys.argv[2])]["new"])' "$spec" $k)"
      python3 "$ONCE" "$file" "$old" "$new" >/dev/null 2>&1; rc_last=$?
      [[ $rc_last -ne 0 ]] && break
    done
  fi
  echo "name=case case=$name mode=$mode event=$event dry_run_rc=$rc_dry last_rc=$rc_last applied=$(applied "$file")/3"
}

run_case C1_each_no_event        each  none
run_case C2_batch_no_event       batch none
run_case S1_each_relabel_between each  relabel_after_first
run_case S2_batch_relabel_between batch relabel_after_first
run_case S3_each_interrupted     each  interrupt_after_first
echo "name=done cases=5"
