#!/usr/bin/env bash
# S7 喂法：用户给了逐台不同的参数（第二台这一趟只给 16 线程、本机照 31）。主 agent 能写进提示的两种写法，派执行员，各版 ⑭ ⑮ 判不判。
#   P-text   「多机：<规范句>」照写，用户的参数另起一行普通正文写（P7 之后子 agent 对这一行「一概不照」）
#   P-valve  「线程上限：16」加「线程上限已判：用户给了第二台 16」（子 agent 照这一个数，「双机时两台各取这一个数」，本机也降到 16）
# 用法：s7_user_per_host.sh <名>=<副本根>... -- <新目录>
set -u
roots=(); while [[ "$1" != -- ]]; do roots+=("$1"); shift; done; shift; work="$1"; mkdir -p "$work"
first="${roots[0]#*=}"
canonical="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import lib_long_run; print(lib_long_run.CANONICAL_HOSTS_SENTENCE)' "$first/.claude/hooks")"
head="开工先读 \`.claude/agent-common.md\`。
重型测试：不跑
草稿目录：$work/fake-scratch/
报告：$work/fake-scratch/report.md
活：两台分片复跑一段崩溃枚举取样，交回数。
这一段回答的岔路：岔路单第 1 行（分片方式）
预计挂钟：超过 2 小时，按上一次同类活推的
分片：按测试目标切，一个目标一片。
续跑：同一条命令再起只补还没有结果文件的目标。"
printf '%s\n' "$head" "线程上限：31" "多机：$canonical" "用户 2026-10-13 给了参数：第二台这一趟只给 16 线程（它上面在跑用户自己的基准），本机照 31。" > "$work/P-text.txt"
printf '%s\n' "$head" "线程上限：16" "线程上限已判：用户 2026-10-13 给了参数，第二台这一趟只给 16 线程" "多机：$canonical" > "$work/P-valve.txt"
for name in P-text P-valve; do
  line="$name"
  for pair in "${roots[@]}"; do
    label="${pair%%=*}" root="${pair#*=}"
    CLAUDE_PROJECT_DIR="$root" PYTHONDONTWRITEBYTECODE=1 bash "$root/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$work/$name.txt" --type experiment-runner --root "$root" > "$work/$name.$label.out" 2>&1
    line="$line	$label=$?"
  done
  printf '%s\n' "$line"
done
