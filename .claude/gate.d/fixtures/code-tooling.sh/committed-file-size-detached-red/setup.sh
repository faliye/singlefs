#!/usr/bin/env bash
# committed-file-size 格的乙红样本（游离 HEAD）：场景同 committed-file-size-unpushed-red，最后 git checkout --detach。
# gate.sh --staged 的临时树就是这样：HEAD 游离，@{upstream} 解析不了，要靠「指着 HEAD 的本地分支各自的上游」找到上游「上游」。
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
git checkout -q --detach
