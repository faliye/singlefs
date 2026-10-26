#!/usr/bin/env bash
# committed-file-size 格的乙红样本：门限 1024 字节。上游在基准提交；第一笔没推的提交加了 big.bin（2048 字节），
# 第二笔又把它删了。HEAD 的树里只剩 README.txt，甲没有东西可抓；推的时候第一笔提交里的 blob 照样要上传，乙要点名那笔提交与 big.bin。
# 只红乙：汇总句括号里只有「乙」。
set -e
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
printf '说明\n' > README.txt
git add README.txt
git commit -qm '基准'
git branch 上游 && git branch -q --set-upstream-to=上游 master
head -c 2048 /dev/zero > big.bin
git add big.bin
git commit -qm '加了大文件'
git rm -q big.bin
git commit -qm '又删了大文件'
