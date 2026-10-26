#!/usr/bin/env bash
# committed-file-size 格的没有上游样本：门限 1024 字节，当前分支没设上游，乙这一半没法判：
# 要打「乙（还没推的提交）本次未跑」并说明原因（甲照判，2 个跟踪的文件都不超，edge.bin 恰好 1024 字节），整格退 0，
# 成功句里写「乙 本次未跑」，不报成乙判过。
set -e
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
printf '说明\n' > README.txt
head -c 1024 /dev/zero > edge.bin
git add README.txt edge.bin
git commit -qm '基准'
