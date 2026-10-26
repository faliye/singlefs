#!/usr/bin/env bash
# 把 .claude/agent-common.md「命名检查交回前跑」给的两条命令照抄（另加 nice -n 19 的写法），按三种该跑的类型与 three-way-attack 喂给四道 Bash 钩子，打退出码；
# 对照：点名里夹着 format-constants 的那一条，实现员起要被 heavy-test-guard.sh 拒（退 2），拒不了说明这一组探针没喂进去。
# 用法：hook_probe.sh <仓根> <草稿目录>；只喂钩子，不起命令本身。
set -uo pipefail
repo="$1"; work="$2"; root="$work/copy-root"
cd "$repo"
declare -a commands=(
  "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=交回前核自己改动里的命名' bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names $root"
  "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh $root"
)
input_of() { python3 -c 'import json,sys; print(json.dumps({"tool_name":"Bash","agent_type":sys.argv[1],"agent_id":"aprobe0000000002","session_id":"probe","tool_input":{"command":sys.argv[2]}}))' "$1" "$2"; }
for agent in implementation-writer experiment-runner tooling-writer three-way-attack; do
  for command in "${commands[@]}" "${commands[@]/#/nice -n 19 }"; do
    for hook in heavy-test-guard run-scope-guard two-logs-guard bash-command-detector; do
      input_of "$agent" "$command" | CLAUDE_PROJECT_DIR="$repo" AGENT_HOOK_DETECTIONS="$work/detections.log" bash ".claude/hooks/$hook.sh" > /dev/null 2>&1
      printf '%s\t%s\t%s\trc=%s\n' "$agent" "${command:0:60}" "$hook" "$?"
    done
  done
done
control="bash .claude/gate.d/code-source-discipline.sh --check vague-names,format-constants $root"
input_of implementation-writer "$control" | CLAUDE_PROJECT_DIR="$repo" AGENT_HOOK_DETECTIONS="$work/detections.log" bash .claude/hooks/heavy-test-guard.sh > /dev/null 2>&1
echo "对照（夹着 format-constants）heavy-test-guard rc=$?"
