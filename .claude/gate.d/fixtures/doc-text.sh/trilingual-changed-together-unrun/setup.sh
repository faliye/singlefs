#!/usr/bin/env bash
# 这一轮只改了与登记无关的 other.txt：一组三语文件都没碰到，这一格退 77（本次未跑，不是通过）。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
git init -q .
git config user.email t@example.com; git config user.name t
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是中文版本'
put README.en.md '# Sample'
put README.md '# 样本'
put README.ja.md '# サンプル'
put other.txt 'base'
git add -A && git commit -qm base
printf '%s\n' 'changed' >> other.txt
