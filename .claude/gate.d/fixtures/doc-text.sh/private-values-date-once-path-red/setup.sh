#!/usr/bin/env bash
# 从第二台拷来的一份对拍记录，文件名带第二台的假主机名（样本的 multi-host.env 里 PEER_SSH_HOST，编的），正文一行重复标题日期：date-once 红，红行里的路径打码。
# 不建 git 仓：date-once 整棵 records/ 与 .claude/kb/ 都判。
set -e
mkdir -p records
printf '%s\n' '# 第二台对拍' '' '## 2026-10-13' '' '- 对拍开跑（2026-10-13），第二台照常。' > records/2026-10-13-sample-peer-box-17-对拍.md
