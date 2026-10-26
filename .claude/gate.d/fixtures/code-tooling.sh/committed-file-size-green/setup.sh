#!/usr/bin/env bash
# committed-file-size 格的绿样本：门限被 .large-file-limit 压到 1024 字节（样本不真造 100 MiB 的文件），
# 跟踪的 3 个文件里 edge.bin 恰好 1024 字节（严格大于才算超，等于放行），上游在基准提交、还有 1 笔没推的提交只加了个小文件。
# 甲查 3 个跟踪的文件，乙查相对「上游」还没推的 1 笔提交里新增的 1 个 blob。
set -e
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
printf '说明\n' > README.txt
head -c 1024 /dev/zero > edge.bin
git add README.txt edge.bin
git commit -qm '基准'
git branch 上游 && git branch -q --set-upstream-to=上游 master
printf '又一份小文件\n' > notes.txt
git add notes.txt
git commit -qm '没推的小提交'
