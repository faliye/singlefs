#!/usr/bin/env bash
# 模型 V3（登记目录很大）：临时登记目录里造 N 条已结束（有 exit）或 N 条没留退出码（单元不在）的登记，量 list 一次的耗时与行数。
# 只跑 list（只读）；「没留退出码」那一类每条问一次真的 systemctl --user show（单元名不存在，只读），list-units 也是真的（只读）。
# 用法：bash model-v3-list-scale.sh <detached-run.sh 的路径> <N…>
set -u
script="$1"; shift
for n in "$@"; do
  for kind in ended vanished; do
    state="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-v3.XXXXXX")"
    for i in $(seq "$n"); do
      r="$state/opusr2v3-$kind-$i"; mkdir -p "$r"
      echo "singlefs-detached-opusr2v3-$kind-$i.service" > "$r/unit"; echo 1790000000 > "$r/started"; echo 2026-10-01 > "$r/started-date"
      echo "sleep 1" > "$r/command"; printf '%s\n第 %s 条的最后一行\n' "$r/log" "$i" > "$r/log"
      if [[ $kind == ended ]]; then echo 0 > "$r/exit"; echo 1790000100 > "$r/ended"; echo 2026-10-01 > "$r/ended-date"; fi
    done
    start=$(date +%s%N)
    DETACHED_RUN_STATE_DIRECTORY="$state" bash "$script" list > "$state.out" 2>&1; rc=$?
    end=$(date +%s%N)
    echo "N=$n 类=$kind list 退 $rc 耗时 $(( (end - start) / 1000000 )) ms 输出 $(wc -l < "$state.out") 行 $(wc -c < "$state.out") 字节；末行：$(tail -n 1 "$state.out" | cut -c1-120)"
    rm -rf -- "$state" "$state.out"
  done
done
