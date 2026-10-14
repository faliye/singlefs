#!/usr/bin/env python3
"""把提示按 hook 的 JSON 喂给派发闸的 stdin 入口（PreToolUse:Agent 的形状），退出码 2 是拒；两份冻结副本各喂一遍。
每个用例用自己的草稿目录（stdin 入口放行时会在里面建 .writer-lock）；DISPATCH_GUARD_STATE_ROOT 指到草稿目录里，不碰会话共用的登记。
用法：feed_stdin.py --frozen <新> --old <旧> --work <草稿目录>"""
import argparse, json, os, subprocess

ap = argparse.ArgumentParser()
ap.add_argument("--frozen", required=True)
ap.add_argument("--old", required=True)
ap.add_argument("--work", required=True)
args = ap.parse_args()

def prompt_for(kind, scratch, thread, wall="预计挂钟：2 小时内"):
    inputs = {"clerk": f"草稿目录：{scratch}/\n报告：{scratch}/r.md\n",
              "three-way-forward": f"禁读清单：无\n草稿目录：{scratch}/\n背景材料：x\n报告：{scratch}/r.md\n",
              "three-way-defense": f"禁读清单：无\n草稿目录：{scratch}/\n背景材料：x\n判决：x\n"}[kind]
    return "重型测试：不跑\n开工先读 `.claude/agent-common.md`\n" + inputs + (wall + "\n" if wall else "") + (thread + "\n" if thread else "")

CASES = [("clerk", "线程上限：31（第二台十六）", "预计挂钟：2 小时内"), ("clerk", "线程上限：31（本机）", "预计挂钟：2 小时内"), ("clerk", "", "预计挂钟：2 小时内"),
         ("clerk", "线程上限：31（<登记值>）", "预计挂钟：2 小时内"), ("three-way-forward", "线程上限：31", ""), ("three-way-forward", "线程上限：31（统一取登记值）", "预计挂钟：2 小时内"),
         ("three-way-forward", "线程上限：31", "预计挂钟：2 小时内"),
         ("clerk", "线程上限：31\n线程上限说明：本机，登记值", "预计挂钟：2 小时内"), ("clerk", "线程上限：31\n线程上限已判：这件活只读", "预计挂钟：2 小时内"),
         ("clerk", "线程上限已判：这件活只读", "预计挂钟：2 小时内"), ("three-way-defense", "线程上限：31（本机）", "预计挂钟：2 小时内")]
for index, (kind, thread, wall) in enumerate(CASES):
    row = []
    for label, root in (("新", args.frozen), ("旧", args.old)):
        scratch = os.path.join(args.work, f"stdin-{label}-{index}")
        os.makedirs(scratch, exist_ok=True)
        text = prompt_for(kind, scratch, thread, wall)
        payload = json.dumps({"hook_event_name": "PreToolUse", "tool_name": "Agent", "tool_input": {"subagent_type": kind, "prompt": text}, "cwd": root})
        env = dict(os.environ, DISPATCH_GUARD_STATE_ROOT=os.path.join(args.work, "state"), CLAUDE_PROJECT_DIR=root)
        done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh")], input=payload, capture_output=True, text=True, cwd="/", env=env, timeout=120)
        first = next((line.strip() for line in done.stderr.split("\n") if line.strip().startswith("✗")), "-")
        row.append(f"{label} 退出={done.returncode} {first[:110]}")
    print(f"{kind} | 线程={thread or '(无)'} | 挂钟={wall or '(无)'}\n    " + "\n    ".join(row))
