#!/usr/bin/env python3
"""E1 第二件：同一批换写法，喂给别的重型类（广谱变异驱动、checker 档 cargo test），分清哪几种只漏 Verus 判法脚本、哪几种各类共有。
只喂 JSON、看退出码。符号链接先在草稿目录里建好（广谱变异的认法读盘），被判的命令一条都不执行。
用法：python3 e1b-shared-shapes.py <仓根> <草稿目录>；每行「退出码<TAB>类<TAB>编号 说明」（实现员身份，2 = 拒、0 = 放）。"""
import json, os, subprocess, sys

root, work = sys.argv[1], sys.argv[2]
links = os.path.join(work, "links")
os.makedirs(links, exist_ok=True)
hook = os.path.join(root, ".claude/hooks/heavy-test-guard.sh")
vpc_name, bm_name = "verus-proof-check" + ".py", "broad-mutation" + ".py"
for name in (vpc_name, bm_name):
    for link in (os.path.join(links, name), os.path.join(links, "renamed-" + name)):
        if os.path.lexists(link):
            os.unlink(link)
        os.symlink(os.path.join(root, "research", "scripts", name), link)
checker = "singlefs-" + "checker-tier"
shapes = [
    ("S0 原样", "python3 research/scripts/{s} {a}", "cargo test -p {c}"),
    ("S1 env -C 换目录", "env -C research/scripts python3 {s} {a}", "env -C crates cargo test -p {c}"),
    ("S2 PYTHONPATH= 前缀再 -m", "PYTHONPATH=research/scripts python3 -m {m} {a}", None),
    ("S3 python3 -c runpy.run_path", "python3 -c \"import runpy; runpy.run_path('research/scripts/{s}', run_name='__main__')\" {a}", None),
    ("S4 仓外同名符号链接（先建好）", "python3 " + links + "/{s} {a}", None),
    ("S5 仓外改名符号链接（先建好）", "python3 " + links + "/renamed-{s} {a}", None),
    ("S6 脚本目录放变量", "R=research/scripts; python3 $R/{s} {a}", None),
    ("S7 script -qc", "script -qc \"python3 research/scripts/{s} {a}\" /dev/null", "script -qc \"cargo test -p {c}\" /dev/null"),
    ("S8 参数放变量", "A='{a}'; python3 research/scripts/{s} $A", "P={c}; cargo test -p $P"),
]
kinds = [("Verus 判法", vpc_name, "must-red-worlds ."), ("广谱变异", bm_name, "--shard 0/2")]
for label, python_form, cargo_form in shapes:
    commands = [(kind, python_form.format(s=script, m=script[:-3], a=arguments)) for kind, script, arguments in kinds]
    if cargo_form:
        commands.append(("checker 档 cargo test", cargo_form.format(c=checker)))
    for kind, command in commands:
        hook_input = {"tool_name": "Bash", "cwd": root, "session_id": "e1b-probe", "agent_type": "implementation-writer",
                      "tool_input": {"command": command, "run_in_background": False}}
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=os.path.join(work, "e1b-detections.jsonl"))
        environment.pop("SINGLEFS_HEAVY_TESTS", None)
        completed = subprocess.run(["bash", hook], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, cwd=root)
        print(f"{completed.returncode}\t{kind}\t{label}")
