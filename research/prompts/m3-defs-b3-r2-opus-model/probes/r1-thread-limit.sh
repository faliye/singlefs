#!/usr/bin/env bash
# R1 探针：派发闸 ⑮ 在几种「线程上限」写法下放不放。喂 JSON 给冻结副本拷贝里的派发闸入口（CLAUDE_PROJECT_DIR 指样本根）。
# 用法：bash r1-thread-limit.sh <冻结副本> <新的工作目录>；每格一行 name=r1 case=<格> rc=<退出码> thread_msg=<stderr 里提没提「线程上限」>
set -euo pipefail
FROZEN="$1"; WORK="$2"
ROOT="$WORK/r1-root"
mkdir -p "$ROOT/research/prompts"
rsync -a --exclude __pycache__ "$FROZEN/.claude" "$FROZEN/scripts" "$ROOT/"
feed() {   # feed <格名> <类型> <提示里线程那几行>
  local label="$1" type="$2" lines="$3" rc
  python3 - "$type" "$lines" > "$WORK/r1-$label.json" <<'PY'
import json, sys
kind, lines = sys.argv[1], sys.argv[2]
common = "重型测试：不跑\n开工先读 `.claude/agent-common.md`\n预计挂钟：2 小时内\n"
bodies = {
    "tooling-writer": "改一份脚本的拒绝说明。要改的文件：scripts/hosts/capped.sh。出口：自证绿。报告放在本次的草稿目录里（草稿目录在工作根下）。\n",
    "clerk": "把一个 harness 档的慢档用例编出来跑一遍，报告放在本次的草稿目录里（草稿目录在工作根下）。\n",
    "three-way-defense": "复核第一轮判决，判决在 research/prompts/x-r1-main-verification.md；背景材料在 research/prompts/_x-r2-background.md；禁读清单：无；报告放在本次的草稿目录里（草稿目录在工作根下）。要编译的拷进草稿目录编。\n",
}
print(json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": kind, "prompt": bodies[kind] + lines + common}}, ensure_ascii=False))
PY
  if env -u SINGLEFS_THREAD_CAP CLAUDE_PROJECT_DIR="$ROOT" bash "$ROOT/.claude/hooks/runner-dispatch-guard.sh" < "$WORK/r1-$label.json" > "$WORK/r1-$label.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=r1 case=$label type=$type rc=$rc thread_msg=$(grep -c '线程上限' "$WORK/r1-$label.out" || true)"
}
# 对照
feed ctl-31 tooling-writer $'线程上限：31\n'
feed ctl-31-note tooling-writer $'线程上限：31（本机）\n'
feed ctl-second-host-digit tooling-writer $'线程上限：31（本机），第二台 16\n'
feed ctl-no-line tooling-writer ''
# 括注里写中文数字、减半、两台合计（括注里没有阿拉伯数字、没有 ÷ / 一半 除以）
feed note-second-host-chinese-number tooling-writer $'线程上限：31（本机；第二台十六）\n'
feed note-second-host-halved tooling-writer $'线程上限：31（第二台减半）\n'
feed note-two-hosts-total tooling-writer $'线程上限：31（两台合计）\n'
feed note-fraction-words tooling-writer $'线程上限：31（实际用三分之一）\n'
# 不是 <填 …> 的尖括号占位：⑬ 不认、⑮ 当占位跳过，缺行那一条也不判
feed angle-placeholder tooling-writer $'线程上限：<登记值>\n'
feed angle-placeholder-any tooling-writer $'线程上限：<按机器忙闲定>\n'
# 第二台另起一行，但标签不是行首「线程上限：」
feed second-line-prefixed tooling-writer $'线程上限：31\n第二台线程上限：16\n'
feed second-line-bracket-label tooling-writer $'线程上限：31\n线程上限（第二台）：16\n'
feed second-line-backtick tooling-writer $'线程上限：31\n`线程上限：16`（第二台照这一行）\n'
# 已判行：理由写「机器忙」（用户定案：机器忙也固定 31）
feed valve-machine-busy tooling-writer $'线程上限：8\n线程上限已判：机器忙，别的会话在跑提交阶段，让出 CPU\n'
feed valve-any-reason tooling-writer $'线程上限：8\n线程上限已判：见上\n'
# 不在类型表里、定义许它编译跑代码的类型：不写这一行
feed clerk-compile-no-line clerk ''
feed defense-compile-no-line three-way-defense ''
# 全角数字、上标（认不出数，拒：不算绕过）
feed fullwidth-digits tooling-writer $'线程上限：３１\n'
feed superscript-digits tooling-writer $'线程上限：³¹\n'
