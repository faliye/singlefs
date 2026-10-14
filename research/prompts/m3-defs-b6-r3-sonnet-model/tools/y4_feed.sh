#!/usr/bin/env bash
# Y4 / V2：喂派发闸 --check-prompt，看「线程上限」与「线程上限已判」几种写法的判定。用法：y4_feed.sh <冻结副本拷贝根> <工作目录>
# 只用 .claude/hooks/runner-dispatch-guard.sh 的 --check-prompt 入口，不建锁、不读主会话记录。
set -u
R="$1"; W="$2"; mkdir -p "$W/e"
export PYTHONDONTWRITEBYTECODE=1
base() { printf '重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：%s/e/\n报告：%s/e/r.md\n预计挂钟：2 小时内\n' "$W" "$W"; }
case_run() { # 名 类型 提示正文
  local name="$1" type="$2" body="$3" f="$W/p-$1.md" rc
  { base; printf '%s\n' "$body"; } > "$f"
  (cd "$R" && CLAUDE_PROJECT_DIR="$R" bash .claude/hooks/runner-dispatch-guard.sh --check-prompt "$f" --type "$type" --root "$R" > "$W/o-$name.txt" 2>&1); rc=$?
  printf '%s\t%s\trc=%s\t线程相关行数=%s\t%s\n' "$name" "$type" "$rc" "$(grep -c '线程上限' "$W/o-$name.txt")" "$(grep -m1 -E '线程上限' "$W/o-$name.txt" | cut -c1-120)"
}
case_run a1-registered clerk '线程上限：31'
case_run a2-eight-no-verdict clerk '线程上限：8'
case_run a3-eight-busy-reason clerk $'线程上限：8\n线程上限已判：机器忙，负载高，让出 CPU'
case_run a4-eight-user-reason clerk $'线程上限：8\n线程上限已判：用户在消息里说让出 CPU'
case_run a5-verdict-only clerk '线程上限已判：用户在消息里说让出 CPU'
case_run a6-verdict-x clerk $'线程上限：8\n线程上限已判：x'
case_run a7-verdict-placeholder clerk $'线程上限：8\n线程上限已判：<理由>'
case_run a8-verdict-none clerk $'线程上限：8\n线程上限已判：无'
case_run a9-verdict-reason-with-number clerk $'线程上限已判：用户让出 CPU，子 agent 起重活用 8 线程'
case_run b1-forward-no-line three-way-forward ''
case_run b2-forward-verdict-only three-way-forward '线程上限已判：这件活不起重活'
case_run b3-31-with-verdict clerk $'线程上限：31\n线程上限已判：机器忙'
case_run b4-8-in-codeblock clerk $'```\n线程上限：8\n```'
case_run b5-note-line clerk $'线程上限：31\n线程上限说明：机器忙时按负载降'
