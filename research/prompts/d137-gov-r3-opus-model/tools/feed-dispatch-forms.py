#!/usr/bin/env python3
"""零轮：共用约束「内存包装排不上」一条与主 agent 定义均衡句写的起法，喂冻结副本里的重型测试闸与范围声明闸，看它们收不收。
用法：feed-dispatch-forms.py <冻结副本根> <输出目录>。只喂 JSON、不执行被喂的命令；cwd 放冻结副本根（命令写相对仓根的路径）；
检出记录与上次红的清单写进 <输出目录>。每条样本一个子进程，start_new_session=True 起、超时杀整组。"""
import json
import os
import signal
import subprocess
import sys

root, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
scope_target = "SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 overwrite_in_one_instance; 范围=1 个测试目标; 为什么=核共用约束那一句写的起法'"
merge = "bash scripts/gate/merge-and-verify.sh --skip-apply research/x"
merge_scope = "SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=合入后验证; 范围=整轮; 为什么=核主 agent 定义均衡句写的起法'"
CASES = [
    # (标签, 身份, 命令, 该怎样)
    ("共用约束第 63 行的例子（target 声明在最前、capped.sh 与 run-build-directory.sh 写在 -- 后面）", "three-way-attack",
     f"{scope_target} python3 scripts/hosts/host-dispatch.py --label probe-a -- bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-harness --test overwrite_in_one_instance",
     "放"),
    ("同一条不带声明", "three-way-attack",
     "python3 scripts/hosts/host-dispatch.py --label probe-b -- bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-harness --test overwrite_in_one_instance",
     "拒（target 声明缺）"),
    ("对照：本机经内存包装、不带声明", "three-way-attack",
     "bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-harness --test overwrite_in_one_instance",
     "拒（target 声明缺）"),
    ("共用约束第 63 行：本仓脚本单独一个 --selftest 经 host-dispatch.py", "three-way-attack",
     "python3 scripts/hosts/host-dispatch.py --label probe-c -- python3 scripts/agents/dispatch-prompt.py --selftest", "放"),
    ("主 agent 定义第 53 行：--plan 照抄「这条命令」（声明与前缀写在 -- 后面）", "",
     f"python3 scripts/hosts/host-dispatch.py --plan -- {merge_scope} SINGLEFS_HEAVY_TESTS=user-request {merge}", "放（只查不跑）"),
    ("主 agent：--plan、声明与前缀写在最前面", "",
     f"{merge_scope} SINGLEFS_HEAVY_TESTS=user-request python3 scripts/hosts/host-dispatch.py --plan -- {merge}", "放（只查不跑）"),
    ("主 agent：--plan、什么都不带", "", f"python3 scripts/hosts/host-dispatch.py --plan -- {merge}", "放（只查不跑）"),
]
detections = os.path.join(out, "detections.jsonl")
environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, SINGLEFS_LAST_RUNS_DIRECTORY=os.path.join(out, "last-runs"))
environment.pop("SINGLEFS_HEAVY_TESTS", None)
for number, (label, agent, command, wanted) in enumerate(CASES, 1):
    payload = {"session_id": "probe-d137-gov-r3-attack", "tool_name": "Bash", "hook_event_name": "PreToolUse", "cwd": root,
               "tool_input": {"command": command}}
    if agent:
        payload["agent_type"] = agent
    for hook in ("heavy-test-guard.sh", "run-scope-guard.sh"):
        process = subprocess.Popen(["bash", os.path.join(root, ".claude/hooks", hook)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, text=True, env=environment, cwd=root, start_new_session=True)
        try:
            text, _ = process.communicate(json.dumps(payload), timeout=120)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            text, _ = process.communicate()
            text = "（超时，整组已杀）" + text
        first = " | ".join(line.strip() for line in text.strip().splitlines()[:3])
        print(f"name=dispatch_form case={number} hook={hook} agent={agent or '主 agent'} rc={process.returncode} wanted={wanted} label={label}")
        print(f"  command: {command}")
        if text.strip():
            print(f"  first-lines: {first[:600]}")
