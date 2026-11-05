# 展开成 probe.py 要的 CASES：每条命令 × 两个钩子 × 主 agent 与子 agent
import os, sys
HERE = os.path.dirname(os.path.abspath(__file__)) if "__file__" in globals() else "."
ns = {}
for name in ("everyday.py", "shellish.py"):
    exec(open(os.path.join(HERE, name), encoding="utf-8").read(), ns)
HOOKS = ["heavy-test-guard.sh", "bash-command-detector.sh"]
CASES = []
for groups in (ns["GROUPS"], ns["GROUPS2"]):
    for group, commands in groups.items():
        for command in commands:
            for hook in HOOKS:
                for agent in ("main", "three-way-defense"):
                    CASES.append((group, hook, agent, command))
