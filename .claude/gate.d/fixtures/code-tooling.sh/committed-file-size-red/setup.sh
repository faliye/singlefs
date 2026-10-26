#!/usr/bin/env bash
# committed-file-size 格的甲红样本：门限 1024 字节。old-big.out（2048 字节）早就在基准提交里（上游已有，乙不会再列它），
# staged-big.bin（3000 字节）刚 git add、还没提交：被判的树是 HEAD 加暂存区，两个都超，都要点名。
# 没推的那 1 笔提交只加了个小文件；untracked-big.bin 没跟踪也没暂存，不在被判的树里，不判。
# 只红甲：汇总句括号里只有「甲」。
set -e
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
printf '说明\n' > README.txt
head -c 2048 /dev/zero > old-big.out
git add README.txt old-big.out
git commit -qm '基准'
git branch 上游 && git branch -q --set-upstream-to=上游 master
printf '又一份小文件\n' > ok.txt
git add ok.txt
git commit -qm '没推的小提交'
head -c 3000 /dev/zero > staged-big.bin
git add staged-big.bin
head -c 5000 /dev/zero > untracked-big.bin
