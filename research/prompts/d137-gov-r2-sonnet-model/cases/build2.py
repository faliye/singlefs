import os
HERE = os.path.dirname(os.path.abspath(__file__))
ns = {}
exec(open(os.path.join(HERE, "everyday2.py"), encoding="utf-8").read(), ns)
CASES = []
for group, commands in ns["GROUPS3"].items():
    for command in commands:
        for hook in ("heavy-test-guard.sh", "bash-command-detector.sh"):
            for agent in ("main", "three-way-defense"):
                CASES.append((group, hook, agent, command))
