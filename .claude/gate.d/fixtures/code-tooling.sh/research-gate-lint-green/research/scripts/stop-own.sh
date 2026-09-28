#!/usr/bin/env bash
# 样本：只停自己点名的一个；按 cgroup 批量发的那一段先核过是自己开的 scope 再标 own-scope（这个文件只拿来扫，不执行）。
sleep 30 &
worker=$!
kill -0 "$worker" && kill -TERM "$worker"
python3 .claude/singlefs-ai-sop/scripts/proc.py stop "$worker"
group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"
case "$group" in */singlefs-memory-cap-*.scope) ;; *) exit 0 ;; esac
while read -r process_id; do kill -TERM "$process_id"; done < "$group/cgroup.procs"  # process-safety:own-scope 上一行 case 核过是自己开的 scope
