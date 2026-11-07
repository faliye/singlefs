#!/usr/bin/env bash
# 样本：内存包装只读上限来记录，不拿它定并发
group="$(cut -d: -f3 /proc/self/cgroup)"
limit="$(cat "/sys/fs/cgroup$group/memory.max")"
RUN_WITH_MEMORY_CAP_KEY=sample SINGLEFS_THREAD_CAP=2 true
printf '%s\n' "$limit"
