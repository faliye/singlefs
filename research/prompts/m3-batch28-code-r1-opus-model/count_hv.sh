#!/usr/bin/env bash
# 数 H-V 那一格：条目不改、另一类那一份在位置条目 0 或 1、违例放在本类那一份里的那几形，目标那一条漏判（NA 或 H）与红的格数。
#   bash count_hv.sh <工作目录>     读 <工作目录>/logs/hv-{new,old,fix}.log；不读 SINGLEFS_REPO
# 不计的两形：墓碑、谱系的「记录数写 0」（I-9.13 只在 inode 叶上判，三份代码不做 H-V 时也是 H）。
set -euo pipefail
logs="${1:?要给工作目录}/logs"
for c in new old fix; do
    rows() { grep -oE 'RESULT HV .*' "$logs/hv-$c.log" | grep 'entry=keep' | grep -E 'other_class_copy=loc[01]' | grep -vE 'target=-:|tree=(tomb|lineage) violation=empty'; }
    echo "$c: 违例在本类那一份、目标漏判（NA 或 H）的格数 = $(rows | grep -cE 'target=I-[0-9.]+:(NA|H) ')；目标红的格数 = $(rows | grep -cE 'target=I-[0-9.]+:V ')"
done
