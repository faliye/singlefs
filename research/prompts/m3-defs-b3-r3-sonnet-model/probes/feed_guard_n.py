#!/usr/bin/env python3 -I
"""U01：capped.sh N 的 N 与派发提示不比、不经 capped.sh 起 cargo 也只看内存包装——把命令喂给第三轮冻结副本的重型测试闸入口。
用法：feed_guard_n.py <冻结副本> <草稿目录>   喂 JSON（agent_type=implementation-writer）给 .claude/hooks/heavy-test-guard.sh；
只看退出码（0 放行、2 拒）。不起 cargo：钩子只读命令行。命令串写在这个脚本里，不写在 bash 命令行上。"""
import json, os, subprocess, sys
frozen, scratch = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
os.makedirs(scratch, exist_ok=True)
CASES = [
  ("经内存包装、capped.sh 31（等于提示里的 31）", "bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 31 cargo test -p singlefs-harness --lib"),
  ("经内存包装、capped.sh 8（提示里是 31）", "bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 8 cargo test -p singlefs-harness --lib"),
  ("经内存包装、不经 capped.sh", "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib"),
  ("既不经内存包装也不经 capped.sh", "cargo test -p singlefs-harness --lib"),
]
for label, cmd in CASES:
    payload = {"tool_name": "Bash", "cwd": frozen, "tool_input": {"command": cmd}, "agent_type": "implementation-writer"}
    env = dict(os.environ, CLAUDE_PROJECT_DIR=frozen, AGENT_HOOK_DETECTIONS=os.path.join(scratch, "det.jsonl"))
    r = subprocess.run(["bash", os.path.join(frozen, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(payload), capture_output=True, text=True, env=env, cwd=frozen)
    print(f"name=feed_guard_n case={label} rc={r.returncode}")

for f in (".claude/hooks/heavy-test-guard.sh", ".claude/hooks/lib_heavy_tests.py", ".claude/hooks/runner-dispatch-guard.sh"):
    n = open(os.path.join(frozen, f), encoding="utf-8").read().count("线程上限")
    print(f"name=feed_guard_n file={f} 线程上限出现次数={n}")
