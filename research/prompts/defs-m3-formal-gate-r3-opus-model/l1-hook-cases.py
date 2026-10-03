#!/usr/bin/env python3
"""L1：G9 改后的认法与只读名单。只把命令写成 hook 的 JSON 喂给 .claude/hooks/heavy-test-guard.sh、看退出码，被判的命令一条都不执行。
用法：python3 l1-hook-cases.py <仓根> <草稿目录>
每行打「退出码<TAB>期望<TAB>身份<TAB>编号 说明」；期望 2 = 该拒（干活），0 = 该放，- = 只观测。末行打对不上期望的条数。"""
import json, os, subprocess, sys

root, work = sys.argv[1], sys.argv[2]
os.makedirs(work, exist_ok=True)
hook = os.path.join(root, ".claude/hooks/heavy-test-guard.sh")
vpc = "research/scripts/" + "verus-proof-check.py"      # 拆开写：本文件不是给闸读的命令
pin = "research/scripts/" + "e165-pin-red-examples.py"
bm = "research/scripts/" + "broad-mutation.py"
cases = [
    # P：preflight 先摘掉每一个 --force，闸的 e165_pin_mode 不摘（期望 2：真跑的是 --replay）
    ("P1 --force 占 --target 的值位，后面写 --check", f"python3 {pin} --replay --target --force --check", 2),
    ("P2 同上带 --root", f"python3 {pin} --root . --replay --target --force --check", 2),
    ("P3 对照：--force 写在末尾", f"python3 {pin} --replay --force", 2),
    ("P4 对照：--check 带 --force", f"python3 {pin} --check --force", 0),
    ("P5 verus-proof-check 首参前写 --force（闸更严，期望拒）", f"python3 {vpc} --force tool-versions .", "-"),
    # C：python -c 只看代码之后的词
    ("C1 -c 里改 sys.argv 成 proof，代码后写 tool-versions",
     f"python3 -c 'import runpy,sys; sys.argv=[\"v\",\"proof\",\".\"]; runpy.run_path(\"{vpc}\", run_name=\"__main__\")' tool-versions", 2),
    ("C2 -c 里改 sys.argv 成 --replay，代码后不写词",
     f"python3 -c 'import runpy,sys; sys.argv=[\"p\",\"--replay\"]; runpy.run_path(\"{pin}\", run_name=\"__main__\")'", 2),
    ("C3 -c 里改 sys.argv 成 --replay，代码后写 --check",
     f"python3 -c 'import runpy,sys; sys.argv=[\"p\",\"--replay\"]; runpy.run_path(\"{pin}\", run_name=\"__main__\")' --check", 2),
    ("C4 -c 里 __import__ 按模块名导入再调 main",
     "python3 -c 'import sys; sys.path.insert(0,\"research/scripts\"); m=__import__(\"verus-proof-check\"); sys.exit(m.main([\"proof\",\".\"]))'", 2),
    ("C5 -c 里 importlib.import_module 起红例重放",
     "python3 -c 'import importlib,sys; sys.path.insert(0,\"research/scripts\"); sys.exit(importlib.import_module(\"e165-pin-red-examples\").main([\"--replay\"]))'", 2),
    ("C6 -c 里路径拼出来", "python3 -c 'import runpy; runpy.run_path(\"research/scripts/verus-\"+\"proof-check.py\", run_name=\"__main__\")' proof .", 2),
    ("C7 heredoc 喂 python3 -", f"python3 - <<'X'\nimport runpy,sys\nsys.argv=['v','proof','.']\nrunpy.run_path('{vpc}', run_name='__main__')\nX", 2),
    ("C8 -c 里调函数 check_proof（spec_from_file_location 点了文件名），代码后写 toolchain",
     f"python3 -c 'import importlib.util as u; s=u.spec_from_file_location(\"v\",\"{vpc}\"); m=u.module_from_spec(s); s.loader.exec_module(m); m.run_check(\"proof\",\".\",m.parse_options([])[1])' toolchain", 2),
    ("C9 对照：-c exec 文件、代码后写 proof", f"python3 -c \"exec(open('{vpc}').read())\" proof .", 2),
    ("C10 对照：python3 -m runpy 加 PYTHONPATH", "PYTHONPATH=research/scripts python3 -m runpy verus-proof-check proof .", 2),
    # M：广谱变异同形（判分不分辨臂：G9 照它那一套认）
    ("M1 广谱变异：-c 里改 sys.argv，代码后写 --summary",
     f"python3 -c 'import runpy,sys; sys.argv=[\"b\"]; runpy.run_path(\"{bm}\", run_name=\"__main__\")' --summary", 2),
    ("M2 广谱变异：-c 里 __import__ 按模块名", "python3 -c 'import sys; sys.path.insert(0,\"research/scripts\"); __import__(\"broad-mutation\").main()'", 2),
    # F：误拒（期望 0）
    ("F1 python3 -m py_compile 两份判法脚本（共用约束点名的语法检查）", f"python3 -m py_compile {vpc} {pin}", 0),
    ("F2 python3 -c 只读文件做语法检查（文件头写的接受的误拒）", f"python3 -c 'import ast; ast.parse(open(\"{vpc}\").read())'", "-"),
    ("F3 -W 选项加 tool-versions", f"python3 -W ignore {vpc} tool-versions .", 0),
    ("F4 同目录名字相近的研究脚本", "python3 research/scripts/verus-proof-check-notes.py proof .", 0),
    ("F5 -c 代码里只在字符串里写了 e165 的名字、代码后不写词（照 e165 判成追加）",
     f"python3 -c 'print(\"{pin}\")'", "-"),
    ("F6 grep 读判法脚本", f"grep -n proof {vpc}", 0),
    ("F7 仓副本里的判法脚本 tool-versions", f"python3 {work}/x/research/scripts/verus-proof-check.py tool-versions .", 0),
]
agents = ["implementation-writer", None]
mismatch = 0
detections = os.path.join(work, "l1-detections.jsonl")
for label, command, want in cases:
    for agent in agents:
        hook_input = {"tool_name": "Bash", "cwd": root, "session_id": "l1-probe", "tool_input": {"command": command, "run_in_background": False}}
        if agent:
            hook_input["agent_type"] = agent
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections)
        environment.pop("SINGLEFS_HEAVY_TESTS", None)
        completed = subprocess.run(["bash", hook], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, cwd=root)
        flag = "" if want == "-" or want == completed.returncode else "  <- 与期望不同"
        if flag:
            mismatch += 1
        print(f"{completed.returncode}\t{want}\t{agent or '主 agent'}\t{label}{flag}")
print(f"对不上期望的：{mismatch} 条（期望写 - 的是待观测的，不计）")
