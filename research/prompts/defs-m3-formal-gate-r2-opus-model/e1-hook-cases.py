#!/usr/bin/env python3
"""E1：判法脚本的干活用法换个写法，闸还认不认。只把命令写成 hook 的 JSON 喂给 .claude/hooks/heavy-test-guard.sh、看退出码，
被判的命令一条都不执行。用法：python3 e1-hook-cases.py <仓根> <草稿目录>
每行打「退出码<TAB>期望<TAB>agent<TAB>编号 说明」；期望 2 = 该拒（干活用法），0 = 该放，- = 只观测。末行打对不上期望的条数。"""
import json, os, subprocess, sys

root, work = sys.argv[1], sys.argv[2]
os.makedirs(work, exist_ok=True)
hook = os.path.join(root, ".claude/hooks/heavy-test-guard.sh")
vpc = "research/scripts/" + "verus-proof-check.py"      # 拆开写：本文件不是给闸读的命令
pin = "research/scripts/" + "e165-pin-red-examples.py"
link_dir = os.path.join(work, "links")
cases = [
    # 对照：今天的认法该认出的（期望 2）
    ("C1 原样", f"python3 {vpc} must-red-worlds .", 2),
    ("C2 cd 进去再 -m", f"cd research/scripts && python3 -m verus-proof-check must-red-worlds ../..", 2),
    ("C3 -m 带包路径", f"python3 -m research.scripts.verus-proof-check must-red-worlds .", 2),
    ("C4 直接执行（带执行位）", f"./{vpc} must-red-worlds .", 2),
    ("C5 bash -c 包一层", f"bash -c 'python3 {vpc} vacuity .'", 2),
    ("C6 timeout 前缀", f"timeout 600 python3 {vpc} proof .", 2),
    ("C7 子 shell 里 cd", f"(cd research/scripts; python3 verus-proof-check.py proof ../..)", 2),
    ("C8 capped.sh 包一层起红例重放", f"bash research/scripts/capped.sh 4 python3 {pin} --replay", 2),
    ("C9 /usr/bin/time -v", f"/usr/bin/time -v python3 {vpc} proof .", 2),
    ("C10 heredoc 喂 bash", f"bash <<'X'\npython3 {vpc} proof .\nX", 2),
    ("C11 内存包装 auto", f"bash research/scripts/run-with-memory-cap.sh auto python3 {pin} --replay", 2),
    ("C12 python3 -I（不带值的选项）", f"python3 -I {vpc} proof .", 2),
    # 换写法（期望 2：都是干活用法）
    ("B1 env -C 换目录", f"env -C research/scripts python3 verus-proof-check.py must-red-worlds ../..", 2),
    ("B2 env --chdir= 换目录", f"env --chdir=research/scripts python3 verus-proof-check.py must-red-worlds ../..", 2),
    ("B3 env PYTHONPATH= 再 -m", f"env PYTHONPATH=research/scripts python3 -m verus-proof-check must-red-worlds .", 2),
    ("B4 PYTHONPATH= 前缀再 -m", f"PYTHONPATH=research/scripts python3 -m verus-proof-check must-red-worlds .", 2),
    ("B5 export PYTHONPATH 再 -m 起红例重放", f"export PYTHONPATH=research/scripts; python3 -m e165-pin-red-examples --replay", 2),
    ("B6 python3 -c runpy.run_path", f"python3 -c \"import runpy; runpy.run_path('{vpc}', run_name='__main__')\" must-red-worlds .", 2),
    ("B7 仓外同名符号链接", f"ln -sf $PWD/{vpc} {link_dir}/verus-proof-check.py && python3 {link_dir}/verus-proof-check.py must-red-worlds $PWD", 2),
    ("B8 仓内改名符号链接", f"ln -sf verus-proof-check.py research/scripts/vpc.py && python3 research/scripts/vpc.py must-red-worlds .", 2),
    ("B9 子命令放变量", f"S=must-red-worlds; python3 {vpc} $S .", 2),
    ("B10 子命令用命令替换", f"python3 {vpc} \"$(echo must-red-worlds)\" .", 2),
    ("B11 shell 函数转参数", f"v() {{ python3 {vpc} \"$@\"; }}; v must-red-worlds .", 2),
    ("B12 红例重放的模式放变量", f"M=--replay; python3 {pin} $M", 2),
    ("B13 脚本目录放变量", f"R=research/scripts; python3 $R/verus-proof-check.py proof .", 2),
    ("B14 xargs 喂子命令（文件头列过的老口子）", f"echo must-red-worlds . | xargs python3 {vpc}", 2),
    ("B15 strace -f", f"strace -f -o /dev/null python3 {vpc} proof .", 2),
    ("B16 flock", f"flock /tmp/claude-1000/v.lock python3 {vpc} proof .", 2),
    ("B17 systemd-run --scope", f"systemd-run --user --scope -q python3 {vpc} proof .", 2),
    ("B18 script -qc", f"script -qc \"python3 {vpc} proof .\" /dev/null", 2),
    ("B19 chrt", f"chrt -i 0 python3 {vpc} proof .", 2),
    ("B20 env -C 起红例重放", f"env -C research/scripts python3 e165-pin-red-examples.py --replay --root ../..", 2),
    ("B21 for 循环逐格跑三格", f"for c in proof vacuity must-red-worlds; do python3 {vpc} $c .; done", 2),
    ("B22 for 循环先核钉全再重放", f"for m in --check --replay; do python3 {pin} $m; done", 2),
    # 只读用法（期望 0）
    ("R1 tool-versions", f"python3 {vpc} tool-versions .", 0),
    ("R2 toolchain", f"python3 {vpc} toolchain .", 0),
    ("R3 红例 --check", f"python3 {pin} --check", 0),
    ("R4 阶段 --tool-versions", "bash .claude/gate.d/checker-tier-" + "formal-verification.sh --tool-versions", 0),
    # 别的重阶段带 --tool-versions：闸按查询放过（阶段自己的参数解析遇到认不出的选项退 2，见报告）
    ("R5 54 号带 --tool-versions", "bash .claude/gate.d/54-" + "layer0-replay.sh --tool-versions", None),
    ("R6 crates 变异整表带 --tool-versions", "bash .claude/gate.d/checker-tier-" + "crates-mutation-replay.sh --tool-versions", None),
]
agents = ["implementation-writer", None]
mismatch = 0
detections = os.path.join(work, "e1-detections.jsonl")
for label, command, want in cases:
    for agent in agents:
        hook_input = {"tool_name": "Bash", "cwd": root, "session_id": "e1-probe", "tool_input": {"command": command, "run_in_background": False}}
        if agent:
            hook_input["agent_type"] = agent
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections)
        environment.pop("SINGLEFS_HEAVY_TESTS", None)
        completed = subprocess.run(["bash", hook], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, cwd=root)
        flag = "" if want is None or want == completed.returncode else "  <- 与期望不同"
        if flag:
            mismatch += 1
        print(f"{completed.returncode}\t{'-' if want is None else want}\t{agent or '主 agent'}\t{label}{flag}")
print(f"对不上期望的：{mismatch} 条（期望写 - 的是待观测的，不计）")
