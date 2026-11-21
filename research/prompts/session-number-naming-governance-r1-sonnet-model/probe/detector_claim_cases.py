#!/usr/bin/env python3
"""共用约束 ㉒ 那一条的文字与检出钩子 ㉒ 的判法逐项对：每条 git 命令只当文本喂给检出钩子（命令不会被执行），读它退不退 2。

用法：detector_claim_cases.py --repo <仓根> --work <工作目录>
  claim 一列是 .claude/agent-common.md 那一条按字面读出来的结局：block 是文字说拒，pass 是文字说放行，silent 是文字没说到。
  hook 一列是真起 bash <仓根>/.claude/hooks/bash-command-detector.sh、输入走标准输入的 PreToolUse JSON 读到的结局。
  检出记录写 <工作目录>，每次起放进自己的进程组。输出每例一行 name=case 起头，末行 name=done。退出码 0 跑完；2 用法错。
"""
import json
import os
import subprocess
import sys

DETECTOR_RELATIVE = ".claude/hooks/bash-command-detector.sh"


def cases(installed):
    return (
        ("commit 带 --no-verify", "git commit --no-verify -m x", "block"),
        ("commit 带缩写 --no-veri", "git commit --no-veri -m x", "block"),
        ("merge 带 --no-verify", "git merge --no-verify topic", "block"),
        ("am 带 --no-verify", "git am --no-verify x.patch", "block"),
        ("push 带 --no-verify", "git push --no-verify origin master", "block"),
        ("commit 带 -n", "git commit -n -m x", "block"),
        ("am 带 -n", "git am -n x.patch", "block"),
        ("push 带 -n（是 --dry-run）", "git push -n origin master", "pass"),
        ("merge 带 -n（是 --no-stat）", "git merge -n topic", "pass"),
        ("命令行 -c 设成别处", "git -c core.hooksPath=/nonexistent commit -m x", "block"),
        ("命令行 -c 设成装对的绝对路径", f"git -c core.hooksPath={installed} commit -m x", "block"),
        ("命令行 --config-env", "git --config-env=core.hooksPath=FOO commit -m x", "block"),
        ("环境里 GIT_CONFIG_PARAMETERS", "GIT_CONFIG_PARAMETERS=\"'core.hooksPath=x'\" git commit -m x", "block"),
        ("环境里 GIT_CONFIG_KEY 加 VALUE 设成装对的", f"GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0={installed} git commit -m x", "block"),
        ("git config 设成别处", "git config core.hooksPath /nonexistent", "block"),
        ("git config 设成装对的绝对路径", f"git config core.hooksPath {installed}", "pass"),
        ("git config 设成装对的相对路径", "git config core.hooksPath scripts/gate/git-hooks", "block"),
        ("git config --add 装对的绝对路径", f"git config --add core.hooksPath {installed}", "silent"),
        ("git config --unset", "git config --unset core.hooksPath", "silent"),
        ("git config 只读", "git config --get core.hooksPath", "pass"),
        ("git config --remove-section core", "git config --remove-section core", "block"),
        ("git config --rename-section core", "git config --rename-section core other", "block"),
        ("别的工具的开关 HUSKY=0", "HUSKY=0 git commit -m x", "pass"),
        ("别名认成 commit", "git -c alias.ci=commit ci -n -m x", "silent"),
        ("启动器里的 commit -n", "echo x | xargs git commit -n", "silent"),
        ("-c 设成别处但子命令是 log", "git -c core.hooksPath=/nonexistent log", "silent"),
    )


def hook_verdict(repo, work, command):
    hook_input = {"hook_event_name": "PreToolUse", "tool_name": "Bash", "session_id": "probe-detector", "cwd": repo, "tool_input": {"command": command, "run_in_background": False}}
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "detections.jsonl"))
    child = subprocess.Popen(["bash", os.path.join(repo, DETECTOR_RELATIVE)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             text=True, env=environment, cwd=repo, start_new_session=True)
    try:
        child.communicate(json.dumps(hook_input, ensure_ascii=False), timeout=120)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, 15)
        child.communicate()
        return "timeout"
    return {0: "pass", 2: "block"}.get(child.returncode, f"exit{child.returncode}")


def main(arguments):
    if len(arguments) != 4 or arguments[0] != "--repo" or arguments[2] != "--work":
        print(__doc__, file=sys.stderr)
        return 2
    repo, work = arguments[1], arguments[3]
    os.makedirs(work, exist_ok=True)
    installed = os.path.join(repo, "scripts", "gate", "git-hooks")
    total = claim_decided_differs = 0
    for label, command, claim in cases(installed):
        total += 1
        hook = hook_verdict(repo, work, command)
        claim_decided_differs += claim != "silent" and hook != claim
        print(f"name=case label={label} claim={claim} hook={hook} command={command}")
    print(f"name=done cases={total} claim_decided_differs_from_hook={claim_decided_differs}")
    return 0


sys.exit(main(sys.argv[1:]))
