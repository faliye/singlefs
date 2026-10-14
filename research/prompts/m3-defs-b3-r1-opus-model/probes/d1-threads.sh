#!/usr/bin/env bash
# D1 探针：冻结副本里的 capped.sh、dispatch-prompt.py 与派发闸 ⑮，在「用户经 SINGLEFS_THREAD_CAP 给了参数」与不给两种环境下各交什么数。
# 用法：bash d1-threads.sh <冻结副本> <新的工作目录>；每格一行 name=d1 …
set -euo pipefail
FROZEN="$1"; WORK="$2"
ROOT="$WORK/d1-root"
mkdir -p "$ROOT"
rsync -a --exclude __pycache__ "$FROZEN/.claude" "$FROZEN/scripts" "$ROOT/"
mkdir -p "$ROOT/research/prompts"
capped="$ROOT/scripts/hosts/capped.sh"
show() {   # show <格名> <命令…>：打那条命令给 RUST_TEST_THREADS 与 SINGLEFS_RANDOM_HISTORY_THREADS 设的值
  local label="$1"; shift
  echo "name=d1 case=$label $(tr '\n' ' ' < <("$@" | grep -E '^(RUST_TEST_THREADS|SINGLEFS_RANDOM_HISTORY_THREADS)='))"
}
show capped-default-no-env env -u SINGLEFS_THREAD_CAP bash "$capped" --print default
show capped-default-env8 env SINGLEFS_THREAD_CAP=8 bash "$capped" --print default
show capped-31-env8 env SINGLEFS_THREAD_CAP=8 bash "$capped" --print 31
# 门禁 checker-tier-layer0-replay 带特性那一趟的前缀就是 capped.sh default（冻结副本该文件第 212 行），GPU 测试二进制拿到的测试线程数：
echo "name=d1 case=layer0-feature-pass-prefix RUST_TEST_THREADS=$(env -u SINGLEFS_THREAD_CAP bash "$capped" default printenv RUST_TEST_THREADS)"
echo "name=d1 case=layer0-gate-mentions-test-threads count=$(grep -c -- '--test-threads' "$FROZEN/.claude/gate.d/checker-tier-layer0-replay.sh" || true)"
# dispatch-prompt.py 的骨架在 SINGLEFS_THREAD_CAP=8 下填什么
echo "name=d1 case=skeleton-env8 $(cd "$ROOT" && env SINGLEFS_THREAD_CAP=8 python3 scripts/agents/dispatch-prompt.py clerk | grep -E '^线程上限')"
# 派发闸 ⑮：喂 JSON 给钩子入口
feed() {   # feed <格名> <环境里的 SINGLEFS_THREAD_CAP，空串是不设> <提示里线程那几行>
  local label="$1" cap="$2" lines="$3" rc
  python3 - "$lines" > "$WORK/d1-$label.json" <<'PY'
import json, sys
p = ("把 records 里一张表抄进草稿目录，报告写进草稿目录里的 report.md。\n" + sys.argv[1] +
     "重型测试：不跑\n开工先读 `.claude/agent-common.md`\n")
print(json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "clerk", "prompt": p}}, ensure_ascii=False))
PY
  if [[ -n "$cap" ]]; then
    if env SINGLEFS_THREAD_CAP="$cap" CLAUDE_PROJECT_DIR="$ROOT" bash "$ROOT/.claude/hooks/runner-dispatch-guard.sh" < "$WORK/d1-$label.json" > "$WORK/d1-$label.out" 2>&1; then rc=0; else rc=$?; fi
  else
    if env -u SINGLEFS_THREAD_CAP CLAUDE_PROJECT_DIR="$ROOT" bash "$ROOT/.claude/hooks/runner-dispatch-guard.sh" < "$WORK/d1-$label.json" > "$WORK/d1-$label.out" 2>&1; then rc=0; else rc=$?; fi
  fi
  echo "name=d1 case=guard15-$label rc=$rc"
}
feed n31 "" $'线程上限：31\n'
feed n16 "" $'线程上限：16\n'
feed n16-valve "" $'线程上限：16\n线程上限已判：用户给了参数 16\n'
feed n8-env8 8 $'线程上限：8\n'
feed n31-env8 8 $'线程上限：31\n'
feed formula-31-div-2 "" $'线程上限：31 ÷ 2\n'
feed n31-second-host-16 "" $'线程上限：31（本机），第二台 16\n'
feed no-line "" ''
