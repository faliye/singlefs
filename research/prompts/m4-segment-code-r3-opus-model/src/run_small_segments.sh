#!/usr/bin/env bash
# 逐个形状跑「不含单元写的段」的崩溃状态（与全落那一个）：参数是一行行「盘宽 单元数 选段 回退目标 形状 批大小」，从标准输入读。
set -u
bin="$1"; cap="$2"; log="$3"
while read -r width units region rb shape k; do
  [[ -z "${width:-}" || "$width" == \#* ]] && continue
  echo "== width=$width units=$units region=$region rb=$rb shape=$shape k=$k" >> "$log"
  ATTACK_ONLY_SEGMENTS_WITHOUT_UNIT_WRITES=1 ATTACK_STATE_LIMIT=100000000 ATTACK_VERBOSE=1 \
  ATTACK_WIDTH="$width" ATTACK_UNITS="$units" ATTACK_REGION="$region" ATTACK_ROLLBACK="$rb" ATTACK_SHAPE="$shape" ATTACK_K="$k" \
    nice -n 19 bash /home/fy5090/.local/share/singlefs/work/m4-predev/scripts/hosts/run-with-memory-cap.sh "$cap" "$bin" --nocapture 2>&1 \
    | grep -E 'OUTCOMES|STREAM|ORDINALS|SUMMARY|PROBLEM|FINDING|ATTACK_TOTAL|panicked|COUNT_ONLY' | sed 's/RecordReuse { [^}]*}//g' | cut -c1-1500 | tail -n 60 >> "$log"
  echo "rc=${PIPESTATUS[0]}" >> "$log"
done
