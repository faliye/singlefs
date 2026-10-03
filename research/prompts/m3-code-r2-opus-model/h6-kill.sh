#!/usr/bin/env bash
# H6：mkfs 打洞用例跑到一半被 SIGTERM 停（测试超时被杀、内存包装限时到点都是这个信号），删除守卫的 Drop 不跑，镜像留在临时目录。
# 只删这一次起的那个进程号的镜像。
set -u
BIN=$1
rm -f /tmp/claude-1000/m3-code-r2-opus/rc/h6-*.rc
cd /tmp/claude-1000/m3-code-r2-opus/work/crates/singlefs-harness
"$BIN" --nocapture --test-threads 1 > /tmp/claude-1000/m3-code-r2-opus/logs/h6-kill-run.log 2>&1 &
child=$!
sleep 6
echo "before-kill pid=$child files:"; ls -ls /tmp/singlefs-step67-punch-zeroed-ranges-"$child"-* 2>&1
kill -TERM "$child"
wait
echo "after-kill files:"; ls -ls /tmp/singlefs-step67-punch-zeroed-ranges-"$child"-* 2>&1
for f in /tmp/singlefs-step67-punch-zeroed-ranges-"$child"-*; do [ -e "$f" ] && rm -f -- "$f" && echo "removed $f"; done
