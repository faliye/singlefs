#!/usr/bin/env bash
# 样本：门禁阶段里按 cgroup.procs 批量发信号、没核 scope 也没标 own-scope；只有 .claude/gate.d/ 进了进程安全的射程才报得出来（只拿来扫，不执行）
group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"
while read -r process_id; do kill -TERM "$process_id"; done < "$group/cgroup.procs"
