#!/usr/bin/env bash
# 建一个最小的 git 仓：判决与材料先提交进 research/prompts/，判决文件再改一次名（内容一字不动，git 的改名检测认得出），最后整批删掉（归档出了树）。
# 决策点名的是改名前的名字 hist-r1-main-verification.md：它只在 git 历史里，名字集合里要有它（它在被加进来的那个提交里列过）。
# 与 evidence-pointer-history-green 只差中间那次改名；弄坏开关 DOC_DECISIONS_BREAK=evidence-history-ignored 不读历史，这份样本判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p research/prompts
printf '# 样本 判决\n' > research/prompts/hist-r1-main-verification.md
for name in _hist-background.md hist-r1-forward-output.md; do
  printf '# 样本\n' > "research/prompts/$name"
done
git add -A && git commit -qm base
git mv research/prompts/hist-r1-main-verification.md research/prompts/hist-r1-main-verification-renamed.md
git commit -qm rename
rm -f research/prompts/*.md
git add -A && git commit -qm archive
