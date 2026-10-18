#!/usr/bin/env bash
# 建一个最小的 git 仓：判决与材料先提交进 research/prompts/，再整批删掉（归档出了树，git 历史里取得回来）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p research/prompts
for name in hist-r1-main-verification.md _hist-background.md hist-r1-forward-output.md; do
  printf '# 样本\n' > "research/prompts/$name"
done
git add -A && git commit -qm base
rm -f research/prompts/*.md
git add -A && git commit -qm archive
