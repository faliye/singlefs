#!/usr/bin/env bash
# admission: always 探针判的是 hook 今天的样子，随时可能改
# run-condition: command python3
# defs-m2-closeout-r1 攻方腿的 hook 探针复跑：只喂 JSON、只看退出码与 stderr 第一行，被判的命令一条都不执行。
# 用法（仓根下）：bash research/prompts/defs-m2-closeout-r1-opus-model/rerun.sh [检出记录目录，默认 ${TMPDIR:-/tmp}]
# 检出记录写进那个目录里新建的临时文件（AGENT_HOOK_DETECTIONS），不进会话共用的那份；复跑完末尾各打一行文件路径，看完自己删。
# 结果依赖仓里此刻的文件：B6/B8b 要 research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out 存在且未进 git。
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
detections_directory="${1:-${TMPDIR:-/tmp}}"
status=0
for cases in cases.json cases2.json cases3.json; do
  TMPDIR="$detections_directory" nice -n 19 python3 "$here/probe.py" "$here/$cases" || status=1
done
exit "$status"
