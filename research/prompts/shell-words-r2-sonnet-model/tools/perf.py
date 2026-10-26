"""perf.py <树目录…> : 几条长命令各喂重型闸一次，打退出码与挂钟秒数（随机器负载变，不进 totals，只看量级）。"""
import sys, json, os, subprocess, time
work = os.environ["SW_ROOT"]
tests = {
    "3000 个收尾的双引号替换后接 bash heredoc": 'echo "$(echo a)"; ' * 3000 + "bash <<'EOF'\ncargo test --all\nEOF",
    "300 层没收尾的 $(": "x=" + "$(" * 300 + "bash <<'EOF'\ncargo test --all\nEOF",
    "300 层没收尾的双引号替换": "x=" + '"$(' * 300 + "bash <<'EOF'\ncargo test --all\nEOF",
    "500 个没配对的双引号": 'echo "; ' * 500 + "bash <<'EOF'\ncargo test --all\nEOF",
    "20 个花括号序列相乘": "echo " + "{1..3}" * 20,
    "引号空串 800 个": "echo " + "'' " * 800 + "; cargo test --all",
    "花括号空份 40 层": "{,}" * 40 + " cargo test --all",
}
for name, command in tests.items():
    cells = []
    for tree in sys.argv[1:]:
        env = dict(os.environ, AGENT_HOOK_DETECTIONS=work + "/det/perf.jsonl", SINGLEFS_WORK_ROOT="/var/empty/swr"); env.pop("SINGLEFS_HEAVY_TESTS", None)
        start = time.time()
        p = subprocess.run(["bash", f"{work}/{tree}/.claude/hooks/heavy-test-guard.sh"], input=json.dumps({"tool_name": "Bash", "cwd": work + "/scratch", "tool_input": {"command": command}}),
                           capture_output=True, text=True, env=env, cwd=work + "/scratch", timeout=200)
        cells.append(f"{tree} rc={p.returncode} {time.time() - start:.2f}s")
    print(f"perf {name}: " + " | ".join(cells))
