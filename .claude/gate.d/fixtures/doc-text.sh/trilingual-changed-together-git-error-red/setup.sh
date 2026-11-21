#!/usr/bin/env bash
# 本该判红：基准里登记的 README 三份齐全，之后 git 配置里的 diff.renames 写成 git 认不得的值，取这次改动碰了哪些路径的 git diff 一律失败。
# 取改动路径是阶段在父进程里预取的，红行之前要有 gate_changed_paths 写的哪一条 git、退几与 git 的报错原文
# （原文里的 diff.renames 是配置项名字，不随 git 的界面语言变）。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
git init -q .
git config user.email t@example.com; git config user.name t
put .claude/trilingual-documents 'README.md  # 样本里的仓根说明，中文为默认版本'
put README.en.md '# Sample'
put README.md '# 样本'
put README.ja.md '# サンプル'
git add -A && git commit -qm base
git config diff.renames foo
