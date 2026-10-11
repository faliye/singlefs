#!/usr/bin/env python3
"""只喂 JSON 给 .claude/hooks/heavy-test-guard.sh，看退出码；被判的命令一条都不执行。
用法：python3 probe.py <仓根> <探测记录文件>"""
import json, os, subprocess, sys
root, detections = sys.argv[1], sys.argv[2]
GATE = "bash .claude/gate.d/checker-tier-formal-verification.sh"
C = "SINGLEFS_HEAVY_TESTS=commit "
U = "SINGLEFS_HEAVY_TESTS=user-request "
CAPS = "GATE_VERUS_MEMORY_MAX=4G GATE_E165_PINNED_MEMORY_MAX=2G "
CV, GT, WR, MA = "crash-verifier", "gate-triage", "implementation-writer", None
# (标签, agent_type, 命令, 期望退出码, 这一格对应哪句字面)
CASES = [
 ("D1-01 崩溃验证员：定义第 2 步的写法（前缀 + nice）", CV, C + "nice -n 19 " + GATE, 0, "crash-verifier.md 第 2 步"),
 ("D1-02 崩溃验证员：定义第 1b 步的写法（两个上限写在 nice 前面、前缀在最前）", CV, C + CAPS + "nice -n 19 " + GATE, 0, "crash-verifier.md 第 1b 步"),
 ("D1-03 崩溃验证员：上限在前、前缀在后", CV, CAPS + C + "nice -n 19 " + GATE, 0, "同上，次序反过来"),
 ("D1-04 崩溃验证员：后台写法 { …; echo exit=$?; } > log", CV, "{ " + C + CAPS + "nice -n 19 " + GATE + '; echo "exit=$?"; } > /tmp/x/formal.log 2>&1', 0, "crash-verifier.md 第 2 步后台写法"),
 ("D1-05 崩溃验证员：外面再包一层内存包装（定义说不再包）", CV, C + "nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 16G " + GATE, 0, "crash-verifier.md 第 1b 步：不再包（闸放不放）"),
 ("D1-06 崩溃验证员：不带前缀", CV, GATE, 2, "heavy-test-guard.sh 文件头：没带一律拒"),
 ("D1-07 崩溃验证员：=user-request 前缀", CV, U + "nice -n 19 " + GATE, 0, "crash-verifier.md 输入：用户要求时 =user-request"),
 ("D1-08 崩溃验证员：单跑一格 --check verus-proof 带前缀", CV, C + GATE + " --check verus-proof", 0, "lib_heavy_tests.py:1327 样本"),
 ("D1-09 崩溃验证员：--list 不带前缀", CV, GATE + " --list", 0, "文件头：--list 不算"),
 ("D1-10 门禁分诊：带前缀直接调这一道", GT, C + GATE, 2, "gate-triage.md 第 2 步：不直接调"),
 ("D1-11 门禁分诊：带前缀单跑一格", GT, C + GATE + " --check verus-proof", 2, "gate-triage.md 第 2 步：单跑某一格也带前缀"),
 ("D1-12 门禁分诊：gate-staged.sh 带前缀", GT, C + "nice -n 19 bash scripts/gate/gate-staged.sh", 0, "gate-triage.md 第 2 步"),
 ("D1-13 门禁分诊：gate.sh 不带参数带前缀", GT, C + "bash .claude/scripts/gate.sh", 0, "gate-triage.md 第 2 步"),
 ("D1-14 主 agent：带前缀整道", MA, C + GATE, 0, "heavy-test-guard.sh 文件头：主 agent 带前缀也能单独跑"),
 ("D1-15 主 agent：不带前缀", MA, GATE, 2, "文件头：主 agent 不带前缀也拒"),
 ("D1-16 实现员：带前缀", WR, C + GATE, 2, "文件头：其余子 agent 一律拒"),
 ("D1-17 实现员：单跑一格 --check verus-proof", WR, GATE + " --check verus-proof", 2, "lib_heavy_tests.py:1327"),
 ("D1-18 实现员：直接起 verus 验自己的证明", WR, "verus --crate-type=lib --no-cheating crates/singlefs-replay-prefix/src/lib.rs", 0, "文件头：直接起的 verus 不算"),
 ("D1-19 实现员：python3 起判法脚本验证明（门禁那一道的判法）", WR, "python3 scripts/gate/verus-proof-check.py proof .", None, "文件头：python 脚本不读正文"),
 ("D1-20 实现员：python3 起判法脚本跑必红世界（94 处 Verus）", WR, "python3 scripts/gate/verus-proof-check.py must-red-worlds .", None, "同上"),
 ("D1-21 实现员：python3 起红例重放（编装置、跑装置）", WR, "python3 scripts/experiments/e165-pin-red-examples.py --replay --root .", None, "同上"),
 ("D1-22 门禁分诊员：python3 起红例重放", GT, "python3 scripts/experiments/e165-pin-red-examples.py --replay --root .", None, "同上"),
 ("D1-24 实现员：--tool-versions（只打 Verus 版本）", WR, GATE + " --tool-versions", None, "按文件名认，除 --list 外一律算"),
 ("D1-25 崩溃验证员：第 1 步的标记核对命令", CV, "python3 scripts/gate/admission.py stage-marker-check . checker-tier-formal-verification.sh", 0, "crash-verifier.md 第 1 步"),
 ("D1-23 崩溃验证员：带前缀起一个读不到的脚本（只看闸怎么判）", CV, C + "bash /tmp/claude-1000/no-such-dir/s.sh", None, "读不到脚本时闸放行并记检出"),
]
def run(agent, command):
    payload = {"tool_name": "Bash", "cwd": root, "tool_input": {"command": command}}
    if agent: payload["agent_type"] = agent
    env = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections)
    done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/heavy-test-guard.sh")], input=json.dumps(payload), capture_output=True, text=True, cwd=root, env=env)
    return done.returncode, (done.stderr or done.stdout).strip().split("\n")[0][:150]
bad = 0
for label, agent, command, expected, basis in CASES:
    code, first = run(agent, command)
    flag = "  " if expected is None or code == expected else "!!"
    if flag == "!!": bad += 1
    print(f"{flag} 退 {code} 期望 {'—' if expected is None else expected} | {label} | agent={agent or '主 agent'} | {first}")
print(f"共 {len(CASES)} 格，期望对不上的 {bad} 格")
