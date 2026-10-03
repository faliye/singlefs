#!/usr/bin/env bash
# 绿样本现场，文件全由这里现写：近似但不是会话号的写法——crate 名、带字母后缀的标识符、上游仓目录名、
# mktemp 模板与路径里的临时目录名、更长的标识符、会话标题（认不出）——都该放行；
# .sh 里带放行标记（写了理由）的一行放行：LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names 时这里判红；
# 落在会话号排除前缀下的门禁样本一份：LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=session-names 时这里判红。
# 这份源码本身不出现会话号：目录名与十六进制之间用 "" 断开。
set -e
repo="single""fs"
mkdir -p records .claude/scripts .claude/gate.d/fixtures/x/red
printf '%s\n' "改 ${repo}-core、${repo}-checker-tier、${repo}-format，上游仓 ${repo}-ai-sop-zh" \
  "mktemp -d \"\${TMPDIR:-/tmp}/${repo}-""e152.XXXXXX\"、缓存在 /tmp/${repo}-""e152-packages、WORK=\${TMPDIR}/${repo}-""e56}" \
  "临时目录 ${repo}-""e129t、${repo}-""e6units、/tmp/claude-1000/${repo}-""6a/progress.md" \
  "交给会话「删除事务和并发事务实现」" > records/近似写法.md
printf '%s\n' "NAME_PATTERN='${repo}-[0-9a-f]+'; echo ${repo}-""6a  # session-names:allow 解析 ListAgents 输出的自检输入" > .claude/scripts/parse.sh
printf '%s\n' "want=交给 ${repo}-""21" > .claude/gate.d/fixtures/x/red/expect
