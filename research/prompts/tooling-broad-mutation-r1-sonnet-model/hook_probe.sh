#!/usr/bin/env bash
# 走真实入口喂 heavy-test-guard.sh：从标准输入给 JSON，读退出码。不跑 --selftest，不起 cargo；检出记录落草稿目录。
cd /home/fy5090/code/singlefs || exit 1
D=/tmp/claude-1000/tooling-broad-mutation-r1-sonnet
S=research/scripts/broad-mutation.py
run() {
  agent="$1"; cmd="$2"
  python3 - "$agent" "$cmd" > "$D/in.json" <<'PY'
import json, sys
d = {"tool_input": {"command": sys.argv[2]}, "cwd": "/home/fy5090/code/singlefs", "session_id": "probe"}
if sys.argv[1] != "-": d["agent_type"] = sys.argv[1]
print(json.dumps(d))
PY
  AGENT_HOOK_DETECTIONS="$D/det.jsonl" bash .claude/hooks/heavy-test-guard.sh < "$D/in.json" > "$D/out.txt" 2> "$D/err.txt"
  echo "agent=$agent exit=$? cmd=$cmd"
}
run - "python3 $S --list-items"
run - "python3 $S --summary"
run - "python3 $S"
run - "SINGLEFS_HEAVY_TESTS=user-request python3 $S"
run - "SINGLEFS_HEAVY_TESTS=user-request nice -n 19 python3 $S --item 12 --skip-stage-two"
run - "python3 $S --selftest"
run implementation-writer "SINGLEFS_HEAVY_TESTS=user-request python3 $S --item 1 --skip-stage-two"
run implementation-writer "python3 $S --item 1 --skip-stage-two"
run implementation-writer "SINGLEFS_HEAVY_TESTS=user-request python3 $S --item 1 --item 2 --item 3 --item 4 --skip-stage-two"
run implementation-writer "SINGLEFS_HEAVY_TESTS=user-request python3 $S --item 1"
run implementation-writer "SINGLEFS_HEAVY_TESTS=user-request python3 $S"
run implementation-writer "SINGLEFS_HEAVY_TESTS=user-request python3 -W ignore $S"
run implementation-writer "cd research/scripts && SINGLEFS_HEAVY_TESTS=user-request python3 -m broad-mutation"
run implementation-writer "python3 $S --sync-tree --commit HEAD"
run implementation-writer "bash research/scripts/multi-host-run.sh -- env SINGLEFS_HEAVY_TESTS=user-request python3 $S"
run implementation-writer "bash research/scripts/multi-host-run.sh -- bash .claude/gate.d/checker-tier-layer0-replay.sh --full ."
run three-way-forward "SINGLEFS_HEAVY_TESTS=user-request python3 $S --selftest"
run crash-verifier "SINGLEFS_HEAVY_TESTS=commit python3 $S"
