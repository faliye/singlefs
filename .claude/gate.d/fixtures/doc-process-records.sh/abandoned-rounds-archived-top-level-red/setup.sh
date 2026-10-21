#!/usr/bin/env bash
# 建一个 git 仓：一个提交里在 research/prompts/ 顶层发了一轮 dropped-r1 的腿（_dropped-r1-body.md、dropped-r1-opus.md），
# 同时在子目录里放一份名字像轮次文件的门禁样本（<轮>-<模型>-model/gate-samples/…/_old-topic-r1-body.md）；下一个提交把两样一起归档删掉。
# dropped-r1 没判决也没登记：只认顶层之后它照判红（只看树会把它整批漏掉）；子目录里那一份不判，红行里不该有 old-topic-r1。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
nested=research/prompts/sample-defs-r1-opus-model/gate-samples/doc-process-records/legacy-dispatch/research/prompts
mkdir -p "$nested"
printf '# 顶层发出去的一轮，后来归档\n' > research/prompts/_dropped-r1-body.md
printf '# dropped-r1 的腿\n' > research/prompts/dropped-r1-opus.md
printf '# 腿造的门禁样本\n' > "$nested/_old-topic-r1-body.md"
git add -A && git commit -qm '发腿与腿的模型目录'
git rm -q research/prompts/_dropped-r1-body.md research/prompts/dropped-r1-opus.md
git rm -rq research/prompts/sample-defs-r1-opus-model
git commit -qm '归档'
