#!/usr/bin/env bash
# V5 喂法：在草稿目录（工作根底下）造「仓副本」wc-*，把 Bash 工具的 hook JSON 喂给 <被判仓>/.claude/hooks/heavy-test-guard.sh 的入口，打印退出码与 stderr 首行。
# 用法：v5_feed.sh <被判仓根> <草稿根>
set -u
repo=$(cd "$1" && pwd); scratch=$(cd "$2" && pwd)
make_copy() {   # $1 名字：拷被判仓的 .claude/hooks（只拷这一层；闸只读副本里的登记表与被起的那份文件）
  local c="$scratch/$1"; rm -rf "$c"; mkdir -p "$c/.claude"; cp -a "$repo/.claude/hooks" "$c/.claude/hooks"; echo "$c"
}
feed() {   # $1 标签 $2 agent_type（空 = 主 agent） $3 cwd $4 命令
  local json
  json=$(python3 -c 'import json,sys; d={"tool_name":"Bash","cwd":sys.argv[2],"tool_input":{"command":sys.argv[3]}}; 
if sys.argv[1]: d["agent_type"]=sys.argv[1]
print(json.dumps(d))' "$2" "$3" "$4")
  local err rc
  err=$(printf '%s' "$json" | CLAUDE_PROJECT_DIR="$repo" bash "$repo/.claude/hooks/heavy-test-guard.sh" 2>&1 >/dev/null); rc=$?
  printf '%s\trc=%s\t%s\n' "$1" "$rc" "$(printf '%s' "$err" | grep -m1 -v '^\s*$')"
}
# wc-lib：heavy-test-guard.sh 与登记表逐字相同，它 import 的 lib_heavy_tests.py 改了（自检里加一条起 checker 档测试的子进程，闸不读正文）
wl=$(make_copy wc-lib)
printf '\n# 副本里改的：自检里起重活\nimport subprocess as _s\n_PLANTED = ["cargo", "test", "-p", "singlefs-checker-tier"]\n' >> "$wl/.claude/hooks/lib_heavy_tests.py"
# wc-words：只改没登记的 lib_shell_words.py
ww=$(make_copy wc-words)
printf '\n# 副本里改的\n_PLANTED = "cargo test --all"\n' >> "$ww/.claude/hooks/lib_shell_words.py"
# wc-body：改登记了的 write-guard.sh 本身
wb=$(make_copy wc-body)
printf '\n# 副本里改的\n' >> "$wb/.claude/hooks/write-guard.sh"
feed S1-lib-imported-by-unchanged-guard tooling-writer "$scratch" "bash $wl/.claude/hooks/heavy-test-guard.sh --selftest"
feed S1b-same-as-implementation-writer implementation-writer "$scratch" "bash $wl/.claude/hooks/heavy-test-guard.sh --selftest"
feed S2-changed-lib-run-directly tooling-writer "$scratch" "python3 $wl/.claude/hooks/lib_heavy_tests.py --selftest"
feed S4-unregistered-lib-imported tooling-writer "$scratch" "bash $ww/.claude/hooks/heavy-test-guard.sh --selftest"
feed S5-changed-body-subagent tooling-writer "$scratch" "bash $wb/.claude/hooks/write-guard.sh --selftest"
feed S6-changed-body-main-agent "" "$scratch" "bash $wb/.claude/hooks/write-guard.sh --selftest"
feed S7-changed-lib-run-directly-main-agent "" "$scratch" "python3 $wl/.claude/hooks/lib_heavy_tests.py --selftest"
