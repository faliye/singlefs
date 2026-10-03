#!/usr/bin/env python3
"""D1：把派发情形写成 hook 的 JSON 喂给 .claude/hooks/heavy-test-guard.sh，只看退出码（被判的命令一条都不执行）。
用法：python3 d1-hook-cases.py <仓根> <草稿目录>；每行打「退出码<TAB>期望<TAB>agent<TAB>说明」，末行打对不上期望的条数。
检出记录写进草稿目录（AGENT_HOOK_DETECTIONS），不进会话的检出日志。"""
import json, os, subprocess, sys

root, work = sys.argv[1], sys.argv[2]
hook = os.path.join(root, ".claude/hooks/heavy-test-guard.sh")
stage = ".claude/gate.d/checker-tier-" + "formal-verification.sh"   # 拆开写：本文件不是给闸读的命令
layer0 = ".claude/gate.d/54-" + "layer0-replay.sh"
commit = "SINGLEFS_HEAVY_TESTS=commit "
caps = "GATE_VERUS_MEMORY_MAX=4G GATE_E165_PINNED_MEMORY_MAX=2G "
cap = "bash research/scripts/run-with-memory-cap.sh "
log = "/tmp/claude-1000/cv-draft/formal.log"
cases = [
    # (说明, agent_type, 命令, run_in_background, 期望)
    ("CV 第 1b/2 步：前缀、两个上限写在 nice 前面", "crash-verifier", commit + caps + "nice -n 19 bash " + stage, False, 0),
    ("CV 第 1b 步原句「写在 nice 前面」另一种排法：上限在前缀前", "crash-verifier", caps + commit + "nice -n 19 bash " + stage, False, 0),
    ("CV 第 2 步后台写法：{ …; echo exit=$?; } > log", "crash-verifier",
     "{ " + commit + caps + "nice -n 19 bash " + stage + "; echo \"exit=$?\"; } > " + log + " 2>&1", True, 0),
    ("CV 第 2 步：与 54 号并行（共用约束 ④ 的写法）", "crash-verifier",
     "rm -f /tmp/claude-1000/cv-draft/b1-*.rc; { " + commit + "nice -n 19 " + cap + "24G bash " + layer0 + "; echo \"$?\" > /tmp/claude-1000/cv-draft/b1-1.rc; } & "
     "{ " + commit + caps + "nice -n 19 bash " + stage + "; echo \"$?\" > /tmp/claude-1000/cv-draft/b1-2.rc; } & wait", True, 0),
    ("CV 第 1 步：先问全绿标记", "crash-verifier", "python3 research/scripts/admission.py stage-marker-check " + root + " checker-tier-formal-verification.sh", False, 0),
    ("CV 单跑一格 --check verus-proof 带前缀", "crash-verifier", commit + "nice -n 19 bash " + stage + " --check verus-proof", False, 0),
    ("CV 单跑一格不带前缀", "crash-verifier", "bash " + stage + " --check verus-proof", False, 2),
    ("CV 照第 1b 步第一句错把这一道也包一层内存包装", "crash-verifier", commit + "nice -n 19 " + cap + "8G bash " + stage, False, 0),
    ("CV 不带前缀打 --tool-versions（只打版本）", "crash-verifier", "bash " + stage + " --tool-versions", False, None),
    ("主 agent 不带前缀打 --tool-versions（只打版本）", None, "bash " + stage + " --tool-versions", False, None),
    ("主 agent 不带前缀打 --list", None, "bash " + stage + " --list", False, 0),
    ("GT 第 2 步：带前缀跑 gate-staged.sh（整轮里含这一道）", "gate-triage", commit + "nice -n 19 bash research/scripts/gate-staged.sh", False, 0),
    ("GT 带前缀单跑这一道", "gate-triage", commit + "nice -n 19 bash " + stage, False, 2),
    ("GT 带前缀单跑一格 --check verus-proof", "gate-triage", commit + "nice -n 19 bash " + stage + " --check verus-proof", False, 2),
    # 绕过：这一道的六格判法是两份 python 脚本，闸不读 python 正文、不按名字认它们
    ("实现员直接跑必红世界格的判法（94 行逐行起 Verus）", "implementation-writer", "python3 research/scripts/verus-proof-check.py must-red-worlds .", False, None),
    ("实现员直接跑空真格的判法", "implementation-writer", "python3 research/scripts/verus-proof-check.py vacuity .", False, None),
    ("实现员直接跑红例重放格的判法（编并跑 checker 档装置）", "implementation-writer", "python3 research/scripts/e165-pin-red-examples.py --replay --root .", False, None),
    ("实现员经内存包装跑红例重放格的判法", "implementation-writer", cap + "4G python3 research/scripts/e165-pin-red-examples.py --replay --root .", False, None),
    ("对照：实现员直接执行同一个装置二进制", "implementation-writer",
     cap + "4G /tmp/singlefs-e165-pinned-target/release/e165_replay_bounded_enumeration --pinned-cases .claude/gate.d/e165-pinned-red-examples.tsv", False, None),
    ("对照：实现员带前缀跑这一道", "implementation-writer", commit + "bash " + stage, False, 2),
    ("主 agent 不带前缀直接跑必红世界格的判法", None, "python3 research/scripts/verus-proof-check.py must-red-worlds .", False, None),
    ("调查员直接跑红例重放格的判法（investigator 第 5 步说这一道不跑）", "investigator", "python3 research/scripts/e165-pin-red-examples.py --replay --root .", False, None),
]
mismatch = 0
detections = os.path.join(work, "d1-detections.jsonl")
for label, agent, command, background, want in cases:
    hook_input = {"tool_name": "Bash", "cwd": root, "session_id": "d1-probe", "tool_input": {"command": command, "run_in_background": background}}
    if agent:
        hook_input["agent_type"] = agent
    environment = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections)
    environment.pop("SINGLEFS_HEAVY_TESTS", None)
    completed = subprocess.run(["bash", hook], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, cwd=root)
    first = (completed.stderr.strip().split("\n") or [""])[0][:150]
    flag = "" if want is None or want == completed.returncode else "  ← 与期望不同"
    if flag:
        mismatch += 1
    print(f"{completed.returncode}\t{'-' if want is None else want}\t{agent or '主 agent'}\t{label}{flag}" + (f"\n\t\t\t  {first}" if completed.returncode else ""))
print(f"对不上期望的：{mismatch} 条（期望写 - 的是待观测的，不计）")
