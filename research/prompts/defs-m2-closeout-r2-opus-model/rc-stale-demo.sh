#!/usr/bin/env bash
# admission: always 演示的是 bash 的作业语义，与仓里的文件无关
# run-condition: command bash
# defs-m2-closeout-r2 云端攻方 F1×F2：攻方第 3b 步要分小批、第 3c 步每件把退出码写进 <草稿目录>/<名字>.rc。
# 两批用同一组名字、第二批的一件在写 .rc 之前被停掉（照共用约束一次停一个自己起的进程），再「按起的次序逐个读 .rc」，读到的是第一批留下的 0。
# 用法：bash research/prompts/defs-m2-closeout-r2-opus-model/rc-stale-demo.sh ；草稿建在 ${TMPDIR:-/tmp} 下，跑完删掉。
set -uo pipefail
work="$(mktemp -d "${TMPDIR:-/tmp}/rc-stale-demo.XXXXXX")"
# 第一批：两件都成功
{ true; echo "$?" > "$work/s1.rc"; } &
{ true; echo "$?" > "$work/s2.rc"; } &
wait
echo "batch1: s1=$(cat "$work/s1.rc") s2=$(cat "$work/s2.rc")"
# 第二批：s1 失败；s2 在写 .rc 之前被停掉
{ false; echo "$?" > "$work/s1.rc"; } &
{ sleep 2; echo "$?" > "$work/s2.rc"; } &
kill -KILL "$!"
wait
echo "batch2: s1=$(cat "$work/s1.rc") s2=$(cat "$work/s2.rc")   # s2 这一件没写，读到的是第一批的"
sleep 3
rm -rf -- "${work:?}"
