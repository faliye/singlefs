#!/bin/bash
# 算 24 步随机历史的写表长度分布，对照持久掩码 2048 位（崩溃注入快档）。
# 输入：主 agent 的探针日志（只读），默认路径如下；输出一行。
log="${1:-$HOME/.local/share/singlefs/work/m3-merge-impl/rh-probe/probe.log}"
steps_per_history=24
grep -o 'item=crash-injection seed=HistorySeed([0-9]*) writes=[0-9]* mkfs_writes=21 after_mkfs=[0-9]*' "$log" \
  | sed 's/.*writes=\([0-9]*\) mkfs_writes.*/\1/' \
  | awk -v steps="$steps_per_history" '
      {w+=$1; n++; if($1>m)m=$1; if(mn==""||$1<mn)mn=$1}
      END{printf "n=%d mean_writes=%.3f max=%d min=%d max_per_step=%.2f mean_per_step=%.2f cap_at_max_rate=%.1f cap_at_mean_rate=%.1f max_scaled_to_60_steps=%.0f\n",
          n, w/n, m, mn, m/steps, w/n/steps, 2048/(m/steps), 2048/(w/n/steps), m*60/steps}'
