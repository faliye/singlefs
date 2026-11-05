"""feedhooks.py <cases.json> <树名> <out.json> [并行数] : 用真的钩子入口逐条喂（bash <钩子>，标准输入给 hook 的 JSON），交回 [{rc, reason}]，与输入同序。
cases 每条 {cmd, hook: guard|det, agent: main|名字, [bg], [cwd: TREE|路径], [env]}；hook 输入的 cwd 默认是草稿目录的 scratch（有 s.sh、light.sh、scripts/hosts/detached-run.sh）。
同 (hook, agent, cmd, bg, cwd, env) 只喂一遍。工作根指到不存在的目录，免得仓副本落在放行范围里。"""
import sys, os, json, subprocess
from concurrent.futures import ThreadPoolExecutor
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.dirname(HERE)
cases = json.load(open(sys.argv[1])); tree = sys.argv[2]; outp = sys.argv[3]; workers = int(sys.argv[4]) if len(sys.argv) > 4 else 16
TREE = f"{ROOT}/trees/{tree}"
FILE = {"guard": "heavy-test-guard.sh", "det": "bash-command-detector.sh"}
def key(c): return (c["hook"], c["agent"], c["cmd"], bool(c.get("bg")), c.get("cwd"), json.dumps(c.get("env") or {}, sort_keys=True))
def feed(k):
    hook, agent, cmd, bg, cwd, envj = k
    ti = {"command": cmd}
    if bg: ti["run_in_background"] = True
    inp = {"tool_name": "Bash", "cwd": TREE if cwd == "TREE" else (TREE + cwd[4:] if (cwd or "").startswith("TREE/") else (cwd or ROOT + "/scratch")), "tool_input": ti}
    if agent != "main": inp["agent_type"] = agent
    env = dict(os.environ, SINGLEFS_WORK_ROOT="/var/empty/swr", AGENT_HOOK_DETECTIONS=f"{ROOT}/out/det-{tree}.jsonl")
    env.pop("SINGLEFS_HEAVY_TESTS", None); env.update(json.loads(envj))
    try:
        p = subprocess.run(["bash", f"{TREE}/.claude/hooks/{FILE[hook]}"], input=json.dumps(inp), capture_output=True, text=True, env=env, cwd=ROOT + "/scratch", timeout=90)
        first = next((l for l in (p.stderr + p.stdout).splitlines() if "✗" in l), "")[:150]
        return dict(rc=p.returncode, reason=first)
    except subprocess.TimeoutExpired:
        return dict(rc="timeout", reason="")
ks = [key(c) for c in cases]; uk = list(dict.fromkeys(ks)); res = {}
with ThreadPoolExecutor(workers) as ex:
    for k, r in zip(uk, ex.map(feed, uk)): res[k] = r
json.dump([res[k] for k in ks], open(outp, "w"), ensure_ascii=False)
from collections import Counter
print(tree, "cases", len(cases), "fed", len(uk), dict(Counter(str(r["rc"]) for r in res.values())))
