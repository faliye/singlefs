"""K3 交回闸 ⑤ 端到端复核：语料里改过 .rs（Edit/Write/MultiEdit 的 file_path 以 .rs 结尾）的实现员、执行员、工具员，
真会话记录（符号链接，不拷不改）放进假主会话目录，交回正文只写「交回」，逐棵树喂 handback-guard.sh 入口，记退出码与第一行 ✗。
不 import 钩子里的任何函数：只看入口的退出码（与工具员在进程内调 naming_problems 的量法不共用代码）。
用法：python3 -I naming_e2e_all.py <语料.jsonl> <项目记录目录> <草稿目录> <仓根（只读，当 CLAUDE_PROJECT_DIR）> <树名=树根>…
输出：每份一行「agent 类型 首条时刻 树名=退出码 …｜第一行 ✗」，末尾按日期分的翻转汇总。"""
import collections, glob, json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
corpus, projects, scratch, root = sys.argv[1:5]
trees = [argument.split("=", 1) for argument in sys.argv[5:]]
KINDS = {"implementation-writer", "experiment-runner", "tooling-writer"}
items = []
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if item["type"] not in KINDS:
        continue
    edited = any(block.get("type") == "tool_use" and block.get("name") in ("Edit", "Write", "MultiEdit")
                 and str(block.get("input", {}).get("file_path", "")).endswith(".rs")
                 for record in item["records"] for block in record["message"]["content"])
    if edited:
        items.append((item["agent"], item["session"], item["type"], item["first_time"] or ""))
base = os.path.join(scratch, "e2e")
os.makedirs(base, exist_ok=False)

def one(entry):
    agent, session, kind, first = entry
    real = glob.glob(os.path.join(projects, session, "subagents", f"agent-{agent}.jsonl"))
    assert len(real) == 1, (agent, real)
    fake = os.path.join(base, agent, "session")
    os.makedirs(os.path.join(fake, "subagents"))
    open(fake + ".jsonl", "w").close()
    os.symlink(real[0], os.path.join(fake, "subagents", f"agent-{agent}.jsonl"))
    hook_input = {"hook_event_name": "PreToolUse", "tool_name": "SubagentHandback", "tool_input": {"message": "交回"},
                  "agent_type": kind, "agent_id": agent, "transcript_path": fake + ".jsonl", "cwd": root}
    verdicts = []
    for name, tree in trees:
        done = subprocess.run(["nice", "-n", "19", "bash", os.path.join(tree, ".claude", "hooks", "handback-guard.sh")], input=json.dumps(hook_input),
                              capture_output=True, text=True, env={**os.environ, "CLAUDE_PROJECT_DIR": root}, timeout=200)
        first_line = next((line for line in done.stderr.split("\n") if line.startswith("✗")), "")
        verdicts.append((name, done.returncode, first_line[:80]))
    return agent, kind, first, verdicts

with ThreadPoolExecutor(max_workers=12) as pool:
    results = sorted(pool.map(one, items), key=lambda result: result[2])
flips = collections.Counter()
for agent, kind, first, verdicts in results:
    print(f"{agent} {kind} {first[:10]} " + " ".join(f"{name}={code}" for name, code, _ in verdicts) + " ｜ " + " ／ ".join(f"{name}:{text}" for name, _, text in verdicts if text))
    codes = tuple(code for _, code, _ in verdicts)
    period = "2026-10-22 起" if first[:10] >= "2026-10-22" else "2026-10-22 之前"
    flips[(period, codes)] += 1
print("汇总（按首条记录的 UTC 日期；退出码依次是 " + "、".join(name for name, _ in trees) + "）：")
for (period, codes), count in sorted(flips.items()):
    print(f"  {period}：{'/'.join(map(str, codes))}：{count} 份")
print(f"共 {len(results)} 份改过 .rs 的")
