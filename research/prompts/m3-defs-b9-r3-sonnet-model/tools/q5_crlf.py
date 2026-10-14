#!/usr/bin/env python3
"""Q5 漏列候选：派发提示用回车换行写，--check-prompt 入口与钩子 stdin 入口对同一份对的提示判得不一样。
用法：q5_crlf.py <名>=<冻结副本根> ... --work <新目录>。只读冻结副本；钩子状态目录指到 --work 里。"""
import argparse, json, os, subprocess
parser = argparse.ArgumentParser(); parser.add_argument("roots", nargs="+"); parser.add_argument("--work", required=True)
arguments = parser.parse_args()
os.makedirs(arguments.work, exist_ok=True)
lf = "\n".join(["# 派发：clerk", "", "重型测试：不跑", "批次：records/x.md#1", "线程上限：31", "草稿目录：@SCRATCH@", "报告：research/prompts/x.md",
                "预计挂钟：2 小时内", "", "## 关的是哪几条", "", "改一个字。", "", "## 出口", "", "交回。", ""])
for entry in arguments.roots:
    label, root = entry.split("=", 1)
    for kind, text in (("lf", lf), ("crlf", lf.replace("\n", "\r\n"))):
        scratch = os.path.join(arguments.work, f"scratch-{label}-{kind}"); os.makedirs(scratch, exist_ok=True)
        body = text.replace("@SCRATCH@", scratch)
        path = os.path.join(arguments.work, f"{label}-{kind}.txt")
        open(path, "w", encoding="utf-8", newline="").write(body)
        env = dict(os.environ, DISPATCH_GUARD_STATE_ROOT=os.path.join(arguments.work, "state"), CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1")
        done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", "clerk", "--root", root],
                              capture_output=True, text=True, env=env, timeout=120)
        payload = json.dumps({"hook_event_name": "PreToolUse", "tool_name": "Agent", "tool_input": {"subagent_type": "clerk", "prompt": body}, "cwd": root})
        hook = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh")], input=payload, capture_output=True, text=True, cwd="/", env=env, timeout=120)
        first = next((l.strip() for l in hook.stderr.split("\n") if l.strip().startswith("✗")), "-")
        print(f"{label}\t{kind}\t--check-prompt 退出={done.returncode}\t钩子 stdin 退出={hook.returncode}\t{first[:60]}")
    # 围栏闭合行缩进 4 格：顶层围栏第二行起是缩进 4 格的三个反引号，后面又有一行真「线程上限：16」和一个收尾围栏
    head = lf.split("## 关的是哪几条")[0]
    for kind, tail in (("闭合行缩进4格+后面一行16+收尾围栏", ["```text", "示例", "    ```", "线程上限：16", "```", ""]), ("正常闭合围栏里的16（对照）", ["```text", "线程上限：16", "```", ""])):
        scratch = os.path.join(arguments.work, f"scratch-{label}-fence"); os.makedirs(scratch, exist_ok=True)
        path = os.path.join(arguments.work, f"{label}-fence.txt")
        open(path, "w", encoding="utf-8", newline="").write((head + "\n".join(tail)).replace("@SCRATCH@", scratch))
        env = dict(os.environ, DISPATCH_GUARD_STATE_ROOT=os.path.join(arguments.work, "state"), CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1")
        done = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", "clerk", "--root", root], capture_output=True, text=True, env=env, timeout=120)
        print(f"{label}\t{kind}\t--check-prompt 退出={done.returncode}")
