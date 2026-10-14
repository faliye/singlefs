#!/usr/bin/env python3
"""Y1 喂法：派发提示里代码块围栏的认法与读者不同（一行里的 ```…``` 被钩子当围栏开头），把真「线程上限：」「多机：」行藏进钩子去掉的那一段。
每个变体经各份副本的 runner-dispatch-guard.sh --check-prompt 判（与钩子入口同一份判法），打印退出码与首行。
用法：y1_fence_feed.py --root <名>=<副本根> ... --work <新目录>"""
import argparse, os, subprocess, sys

parser = argparse.ArgumentParser()
parser.add_argument("--root", action="append", required=True)
parser.add_argument("--work", required=True)
arguments = parser.parse_args()
roots = [entry.split("=", 1) for entry in arguments.root]
os.makedirs(arguments.work, exist_ok=True)
scratch = os.path.join(arguments.work, "fake-scratch")
CANONICAL = "台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"
head = ("开工先读 `.claude/agent-common.md`。\n重型测试：不跑\n草稿目录：" + scratch + "/\n报告：" + scratch + "/report.md\n"
        "活：在 crates/singlefs-harness 里复现一个 harness 测试的失败，交回最小复现。\n现象：harness 的一个测试偶发失败。\n")
within = "预计挂钟：2 小时内\n"
over = "预计挂钟：超过 2 小时，按上一次同类活推的\n分片：按测试目标切，一个目标一片。\n续跑：同一条命令再起只补还没有结果文件的目标。\n"
one_liner = "```cargo test -p singlefs-harness --lib``` 是上一次的起法，照它起。\n"
cases = [
    # (名字, 类型, 提示正文, 读者眼里那一行（只写进输出，便于对照）)
    ("F0-control-31", "investigator", head + within + "线程上限：31\n", "线程上限：31"),
    ("F1-control-bracket-load", "investigator", head + within + "线程上限：31（机器忙，按负载降到 16）\n", "线程上限：31（机器忙，按负载降到 16）"),
    ("F1b-control-16", "investigator", head + within + "线程上限：16\n", "线程上限：16"),
    ("F2-fence-shift-bracket-load", "investigator", head + within + one_liner + "线程上限：31（机器忙，按负载降到 16）\n上一次派发的原文：\n```\n线程上限：31\n```\n",
     "线程上限：31（机器忙，按负载降到 16）（代码块里的 31 是示例）"),
    ("F3-fence-shift-16", "investigator", head + within + one_liner + "线程上限：16\n上一次派发的原文：\n```\n线程上限：31\n```\n",
     "线程上限：16（代码块里的 31 是示例）"),
    ("F4-fence-shift-example-judged", "investigator", head + within + "线程上限：31\n" + one_liner + "上一次派发的原文：\n```\n线程上限：31（本机）\n```\n",
     "线程上限：31（代码块里的「31（本机）」是示例，定义写不判）"),
    ("H0-control-canonical", "investigator", head + over + "线程上限：31\n多机：" + CANONICAL + "\n", "多机：<规范句>"),
    ("H1-control-live", "investigator", head + over + "线程上限：31\n多机：每台线程按此刻空闲现算\n", "多机：每台线程按此刻空闲现算"),
    ("H2-fence-shift-live", "investigator", head + over + "线程上限：31\n" + one_liner + "多机：每台线程按此刻空闲现算，机器忙时按负载降\n骨架打的那一句：\n```\n多机：" + CANONICAL + "\n```\n",
     "多机：每台线程按此刻空闲现算，机器忙时按负载降（代码块里的规范句是示例）"),
    ("H3-fence-shift-half", "investigator", head + over + "线程上限：31\n" + one_liner + "多机：台数取 --emit-hosts，每台线程取登记值的一半\n骨架打的那一句：\n```\n多机：" + CANONICAL + "\n```\n",
     "多机：台数取 --emit-hosts，每台线程取登记值的一半"),
]
for name, kind, text, reader in cases:
    path = os.path.join(arguments.work, name + ".txt")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    cells = []
    for label, root in roots:
        result = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", kind, "--root", root],
                                capture_output=True, text=True, cwd=root, env=dict(os.environ, CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1"))
        first = next((line.strip() for line in (result.stdout + result.stderr).split("\n") if "✗" in line or "✓" in line), "")
        cells.append(f"{label}={result.returncode}")
        if label == roots[0][0]:
            first_line = first[:110]
    print(f"{name}\t{kind}\t" + "\t".join(cells) + f"\t读者看到：{reader}\t{roots[0][0]} 首行：{first_line}")
