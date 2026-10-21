#!/usr/bin/env bash
# 建一个 git 仓：一个提交里在 research/prompts/ 的子目录下放三份名字像轮次文件的样本（腿在自己模型目录里造的门禁样本那种：
# <轮>-<模型>-model/gate-samples/…/research/prompts/_old-topic-r1-body.md、old-topic-r1-opus.md，另一处 _r-victim-r1-body.md），
# 下一个提交把它们归档删掉。git log --diff-filter=D 列得出这三份，但它们不在 research/prompts/ 顶层，不算发过腿：
# abandoned-rounds 只认顶层那一轮 live-r1（有判决文件），判绿。改前取 basename 不限顶层，old-topic-r1 与 r-victim-r1 被当成没判决没登记的轮判红。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
nested=research/prompts/sample-defs-r1-opus-model/gate-samples/doc-process-records
mkdir -p "$nested/legacy-dispatch/research/prompts" "$nested/date-not-iso/research/prompts"
printf '# 腿造的门禁样本：旧形态的一轮\n' > "$nested/legacy-dispatch/research/prompts/_old-topic-r1-body.md"
printf '# 腿造的门禁样本：旧形态的一轮的腿\n' > "$nested/legacy-dispatch/research/prompts/old-topic-r1-opus.md"
printf '# 腿造的门禁样本：登记表写坏的那一轮\n' > "$nested/date-not-iso/research/prompts/_r-victim-r1-body.md"
git add -A && git commit -qm '腿的模型目录与样本'
git rm -rq research/prompts/sample-defs-r1-opus-model && git commit -qm '归档'
