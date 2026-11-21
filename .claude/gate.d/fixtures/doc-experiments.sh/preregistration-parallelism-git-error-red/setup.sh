#!/usr/bin/env bash
# 本该判红：git 配置里的 diff.renames 写成 git 认不得的值，取这次改动碰了哪些路径的 git diff 一律失败。
# 取改动路径是阶段在父进程里预取的，红行之前要有 gate_changed_paths 写的哪一条 git、退几与 git 的报错原文
# （原文里的 diff.renames 是配置项名字，不随 git 的界面语言变）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p research/prompts
printf '# E100 旧登记\n\n## 一、问题\n\n英文名：old_sample\n' > research/prompts/e100-preregistration.md
git add -A
git commit -qm '基准：一份旧登记'
printf '\n## 十二、修订\n\n收严一条门槛。\n' >> research/prompts/e100-preregistration.md
git config diff.renames foo
