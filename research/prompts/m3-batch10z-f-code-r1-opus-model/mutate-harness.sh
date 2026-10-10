#!/usr/bin/env bash
# 攻方草稿：在仓副本 copy-m 上逐条改坏 harness 档一处、跑点名的测试、看红没红、还原。只跑 harness 档（不碰 checker 档）。
# 用法：mutate-harness.sh <变异表> <行号起> <行号止>   表照 crates/mutations.tsv 六段；第六段 * 表示「这一组里任一条红就算抓到」
set -u
# 环境：COPY（冻结副本的一份拷贝，改坏的就是它）、CARGO_TARGET_DIR（那份拷贝自己的编译目录）、LOGS（每条一份日志），FROZEN（还原用）
FROZEN=${FROZEN:-$HOME/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree}
COPY=${COPY:?冻结副本拷贝}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:?编译目录}
LOGS=${LOGS:?日志目录}
mkdir -p "$LOGS"
table=$1; from=$2; to=$3
n=0
while IFS=$'\t' read -r name file old new args must; do
  n=$((n+1)); [[ $n -lt $from || $n -gt $to ]] && continue
  [[ "$name" == \#* || -z "$name" ]] && continue
  cp "$FROZEN/$file" "$COPY/$file"
  if ! python3 - "$COPY/$file" "$old" "$new" <<'PY'
import sys
p, old, new = sys.argv[1], sys.argv[2].replace('\\n', '\n'), sys.argv[3].replace('\\n', '\n')
s = open(p).read()
if s.count(old) != 1:
    print(f"ANCHOR_COUNT={s.count(old)}"); sys.exit(3)
open(p, 'w').write(s.replace(old, new))
PY
  then echo "MUTANT row=$n name=$name result=anchor_broken"; cp "$FROZEN/$file" "$COPY/$file"; continue; fi
  log=$LOGS/row-$n.log
  ( cd "$COPY" && read -ra a <<<"$args" && cargo test --offline "${a[@]}" ) > "$log" 2>&1
  rc=$?
  if grep -q '^error\[' "$log"; then res=compile_error
  elif [[ $rc -ne 0 ]]; then
    if [[ "$must" == "*" ]] || grep -q -- "$must" <(grep -E 'FAILED|panicked' "$log"); then res=caught; else res=red_other_test; fi
  else res=survived; fi
  echo "MUTANT row=$n rc=$rc result=$res name=$name failed=$(grep -c ' FAILED$' "$log")"
  cp "$FROZEN/$file" "$COPY/$file"
done < "$table"
